-- Add migration script here
--
INSERT INTO users (user_id, username, password_hash)
VALUES (
  'ffacacd2-9d5a-442b-ae71-479e37c464d4',
  'admin',
  '$argon2id$v=19$m=15000,t=2,p=1$/9evwws+bFCIWMYWEHcRvw$q6WpVVFno+dpvu3pplswShOvG9KtYAS17U+4MqwWBtQ'
);

