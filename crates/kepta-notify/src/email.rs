use anyhow::Result;
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

pub struct EmailConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
}

pub struct EmailSender {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: String,
}

impl EmailSender {
    pub fn new(cfg: &EmailConfig) -> Result<Self> {
        let creds = Credentials::new(cfg.username.clone(), cfg.password.clone());
        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.host)?
            .port(cfg.port)
            .credentials(creds)
            .build();
        Ok(Self { transport, from: cfg.from.clone() })
    }

    pub async fn send(&self, to: &str, subject: &str, html: &str) -> Result<()> {
        let email = Message::builder()
            .from(self.from.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html.to_string())?;

        self.transport.send(email).await?;
        Ok(())
    }
}
