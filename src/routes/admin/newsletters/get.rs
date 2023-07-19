use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn publish_newsletter_form(
    flash_messages: IncomingFlashMessages
) -> Result<HttpResponse, actix_web::Error> {
    let mut msg_html = String::new();
    for m in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    Ok(
        HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(format!(
                r#"
                <!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta http-equiv="cotent-type" content="text/html; charset=utf-8">
                    <title>Publish newsletter Issue</title>
                </head>
                <body>
                    {msg_html}
                    <form action="/admin/newsletters" method="post">
                        <label>Title:<br>
                            <input
                                type="text"
                                placeholder="Enter the issue title"
                                name="title"
                            >
                        </label>
                        <br/>
                        <label>Plain text content:<br>
                            <input
                                placeholder="Enter the content in plain text"
                                name="text_content"
                                rows="20"
                                cols="50"
                            >
                        </label>
                        <br/>
                        <label>HTML content:<br>
                            <input
                                placeholder="Enter the content in HTML format"
                                name="html_content"
                                rows="20"
                                cols="50"
                            >
                        </label>
                        <br>
                        <button type="submit">Publish</button>
                    </form>
                    <p><a href="/admin/dashboard">&lt; - Back</a></p>
                </body>
                </html>
                "#
            ))
    )
}
