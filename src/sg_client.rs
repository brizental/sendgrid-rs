use errors::SendgridResult;

use mail::Mail;

use std::io::Read;

use reqwest::header::{Authorization, Bearer, ContentType, Headers, UserAgent};
use reqwest::Client;

use url::form_urlencoded::Serializer;

static API_URL: &'static str = "https://api.sendgrid.com/api/mail.send.json?";

/// This is the struct that allows you to authenticate to the SendGrid API.
/// It's only field is the API key which allows you to send messages.
pub struct SGClient {
    api_key: String,
}

// Given a form value and a key, generate the correct key.
fn make_form_key(form: &str, key: &str) -> String {
    let mut value = String::new();
    value.push_str(form);
    value.push('[');
    value.push_str(key);
    value.push(']');

    value
}

// Use the URL form encoder to properly generate the body used in the mail send request.
fn make_post_body(mut mail_info: Mail) -> SendgridResult<String> {
    let body = String::new();
    let mut encoder = Serializer::new(body);

    for to in mail_info.to.iter() {
        encoder.append_pair("to[]", &to);
    }

    for to_name in mail_info.to_names.iter() {
        encoder.append_pair("toname[]", &to_name);
    }

    for cc in mail_info.cc.iter() {
        encoder.append_pair("cc[]", &cc);
    }

    for bcc in mail_info.bcc.iter() {
        encoder.append_pair("bcc[]", &bcc);
    }

    for (attachment, contents) in &mail_info.attachments {
        encoder.append_pair(&make_form_key("files", attachment), contents);
    }

    for (id, value) in &mail_info.content {
        encoder.append_pair(&make_form_key("content", id), value);
    }

    encoder.append_pair("from", &mail_info.from);
    encoder.append_pair("subject", &mail_info.subject);
    encoder.append_pair("html", &mail_info.html);
    encoder.append_pair("text", &mail_info.text);
    encoder.append_pair("fromname", &mail_info.from_name);
    encoder.append_pair("replyto", &mail_info.reply_to);
    encoder.append_pair("date", &mail_info.date);
    encoder.append_pair("headers", &mail_info.make_header_string()?);
    encoder.append_pair("x-smtpapi", &mail_info.x_smtpapi);

    Ok(encoder.finish())
}

impl SGClient {
    /// Makes a new SendGrid cient with the specified API key.
    pub fn new(key: String) -> SGClient {
        SGClient { api_key: key }
    }

    /// Sends a messages through the SendGrid API. It takes a Mail struct as an
    /// argument. It returns the string response from the API as JSON.
    /// It sets the Content-Type to be application/x-www-form-urlencoded.
    pub fn send(self, mail_info: Mail) -> SendgridResult<String> {
        let client = Client::new();
        let mut headers = Headers::new();
        headers.set(Authorization(Bearer {
            token: self.api_key.to_owned(),
        }));
        headers.set(ContentType::form_url_encoded());
        headers.set(UserAgent::new("sendgrid-rs"));

        let post_body = make_post_body(mail_info)?;
        let mut res = client
            .post(API_URL)
            .headers(headers)
            .body(post_body)
            .send()?;
        let mut body = String::new();
        res.read_to_string(&mut body)?;
        Ok(body)
    }
}

#[test]
fn basic_message_body() {
    let mut m = Mail::new();
    m.add_to("test@example.com");
    m.add_from("me@example.com");
    m.add_subject("Test");
    m.add_text("It works");

    let body = make_post_body(m);
    let want = "to%5B%5D=test%40example.com&from=me%40example.com&subject=Test&\
                html=&text=It+works&fromname=&replyto=&date=&headers=%7B%7D&x-smtpapi=";
    assert_eq!(body.unwrap(), want);
}

#[test]
fn test_proper_key() {
    let want = "files[test.jpg]";
    let got = make_form_key("files", "test.jpg");
    assert_eq!(want, got);
}
