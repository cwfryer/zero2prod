use crate::configuration::Settings;
use crate::email_client::EmailClient;
use crate::startup::get_connection_pool;
use crate::domain::SubscriberEmail;
use sqlx::{PgPool, Postgres, Transaction};
use tokio::time::{sleep, Duration};
use tracing::{field::display, Span};
use uuid::Uuid;

const MAX_RETRIES: i16 = 5;

pub enum ExecutionOutcome {
    TaskCompleted,
    EmptyQueue,
}

#[tracing::instrument(
    skip_all,
    fields(
        newsletter_issue_id=tracing::field::Empty,
        subscriber_email=tracing::field::Empty,
        retries=tracing::field::Empty,
    ),
    err
)]
pub async fn try_execute_task(
    pool: &PgPool,
    email_client: &EmailClient,
) -> Result<ExecutionOutcome, anyhow::Error> {
    let task = dequeue_task(pool).await?;
    if task.is_none() {
        return Ok(ExecutionOutcome::EmptyQueue);
    }
    let (transaction, issue_id, email, n_retries, execute_after) = task.unwrap();
    {
        if n_retries == MAX_RETRIES {
            delete_task(transaction, issue_id, &email).await?;
            return Err(anyhow::anyhow!(
                "Failed to deliver issue to a confirmed subscriber. \
                Skipping this subscriber."
            ));
        };
        Span::current()
            .record("newsletter_issue_id", &display(issue_id))
            .record("subscriber_email", &display(&email))
            .record("retries", &display(&n_retries));
        sleep(Duration::from_secs(execute_after as u64)).await;
        match SubscriberEmail::parse(email.clone()) {
            Ok(email) => {
                let issue = get_issue(pool, issue_id).await?;
                if let Err(e) = email_client
                    .send_email(
                        &email,
                        &issue.title,
                        &issue.html_content,
                        &issue.text_content,
                    )
                    .await
                {
                    delete_task(transaction, issue_id, &email.to_string()).await?;
                    requeue_task(
                        pool,
                        issue_id,
                        &email.to_string(),
                        n_retries + 1,
                        execute_after + 1,
                    )
                    .await?;
                    tracing::error!(
                        error.cause_chain = ?e,
                        error.message = %e,
                        "Failed to deliver issue to a confirmed subscriber. \
                        Adding back to the queue.",
                    );
                }
            }
            Err(e) => {
                tracing::error!(
                    error.cause_chain = ?e,
                    error.message = %e,
                    "Skipping a confirmed subscriber. \
                    Their stored credentials are invalid.",
                );
                delete_task(transaction, issue_id, &email).await?;
            }
        }
    }
    Ok(ExecutionOutcome::TaskCompleted)
}

type PgTransaction = Transaction<'static, Postgres>;

#[tracing::instrument(skip_all)]
async fn dequeue_task(
    pool: &PgPool,
) -> Result<Option<(PgTransaction, Uuid, String, i16, i16)>, anyhow::Error> {
    let mut transaction = pool.begin().await?;
    let r = sqlx::query!(
        r#"
        SELECT newsletter_issue_id, subscriber_email, n_retries, execute_after
        FROM issue_delivery_queue
        FOR UPDATE
        SKIP LOCKED
        LIMIT 1
        "#,
    )
    .fetch_optional(&mut transaction)
    .await?;
    if let Some(r) = r {
        Ok(Some((
            transaction,
            r.newsletter_issue_id,
            r.subscriber_email,
            r.n_retries,
            r.execute_after,
        )))
    } else {
        Ok(None)
    }
}

#[tracing::instrument(skip_all)]
async fn delete_task(
    mut transaction: PgTransaction,
    issue_id: Uuid,
    email: &str,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        DELETE FROM issue_delivery_queue
        WHERE
            newsletter_issue_id = $1 AND
            subscriber_email = $2
        "#,
        issue_id,
        email
    )
    .execute(&mut transaction)
    .await?;
    transaction.commit().await?;
    Ok(())
}

struct NewsletterIssue {
    title: String,
    text_content: String,
    html_content: String,
}

#[tracing::instrument(skip_all)]
async fn get_issue(pool: &PgPool, issue_id: Uuid) -> Result<NewsletterIssue, anyhow::Error> {
    let issue = sqlx::query_as!(
        NewsletterIssue,
        r#"
        SELECT title, text_content, html_content
        FROM newsletter_issues
        WHERE
            newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(pool)
    .await?;
    Ok(issue)
}

async fn requeue_task(
    pool: &PgPool,
    issue_id: Uuid,
    email: &str,
    n_retries: i16,
    execute_after: i16,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        INSERT INTO issue_delivery_queue (
            newsletter_issue_id,
            subscriber_email,
            n_retries,
            execute_after
        )
        VALUES (
            $1,
            $2,
            $3,
            $4
        )
        "#,
        issue_id,
        email,
        n_retries,
        execute_after
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn worker_loop(pool: PgPool, email_client: EmailClient) -> Result<(), anyhow::Error> {
    loop {
        match try_execute_task(&pool, &email_client).await {
            Ok(ExecutionOutcome::EmptyQueue) => sleep(Duration::from_secs(10)).await,
            Err(_) => sleep(Duration::from_secs(1)).await,
            Ok(ExecutionOutcome::TaskCompleted) => {}
        }
    }
}

pub async fn run_worker_until_stopped(configuration: Settings) -> Result<(), anyhow::Error> {
    let connection_pool = get_connection_pool(&configuration.database);

    let email_client = configuration.email_client.client();
    worker_loop(connection_pool, email_client).await
}
