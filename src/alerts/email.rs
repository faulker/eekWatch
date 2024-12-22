use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use sysinfo::System;

use crate::config::CONFIG;

pub fn alert(check: &String, msg: String, contacts: &Vec<String>) {
    let config = CONFIG.get().unwrap();

    let to = contacts.join(", ");
    let from = &config.alerts.email.from_address;
    let hostname = System::host_name().unwrap_or("UNKNOWN".to_string());
    let subject = format!("{} - Failed Check: {}", hostname, check);
    let timestamp = chrono::Utc::now().to_rfc2822();

    let email = Message::builder()
        .from(from.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(format!("<h3>{}</h3><b>{}</b>", timestamp, msg))
        .unwrap();

    let smtp_user = config.alerts.email.user.clone();
    let smtp_pass = config.alerts.email.password.clone();

    let creds = Credentials::new(smtp_user, smtp_pass);

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay(&config.alerts.email.smtp)
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {e:?}"),
    }
}
