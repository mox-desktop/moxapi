use std::collections::HashMap;

#[zbus::proxy(
    interface = "org.freedesktop.Notifications",
    default_service = "org.freedesktop.Notifications",
    default_path = "/org/freedesktop/Notifications"
)]
trait Notifications {
    fn get_capabilities(&self) -> zbus::Result<Box<[Box<str>]>>;

    #[allow(clippy::too_many_arguments)]
    async fn notify(
        &self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Box<[&str]>,
        hints: HashMap<&str, zbus::zvariant::Value<'_>>,
        expire_timeout: i32,
    ) -> zbus::Result<u32>;
}

pub struct NotificationManager {
    session_conn: zbus::Connection,
    proxy: NotificationsProxy<'static>,
}

impl NotificationManager {
    pub async fn new() -> anyhow::Result<Self> {
        let session_conn = zbus::Connection::session().await?;
        let proxy = NotificationsProxy::new(&session_conn).await?;

        Ok(Self {
            session_conn,
            proxy,
        })
    }

    pub async fn get_capabilities(&self) -> anyhow::Result<Box<[Box<str>]>> {
        self.proxy
            .get_capabilities()
            .await
            .map_err(|_| anyhow::anyhow!(""))
    }

    pub async fn builder(&self) -> NotificationBuilder {
        NotificationBuilder {
            summary: "",
            body: "",
            proxy: self.proxy.clone(),
            id: 0,
            timeout: 0,
        }
    }
}

pub struct NotificationBuilder<'a> {
    proxy: NotificationsProxy<'static>,
    summary: &'a str,
    body: &'a str,
    timeout: i32,
    id: u32,
}

impl<'a> NotificationBuilder<'a> {
    pub fn with_summary(mut self, summary: &'a str) -> Self {
        self.summary = summary;
        self
    }

    pub fn with_body(mut self, body: &'a str) -> Self {
        self.body = body;
        self
    }

    pub fn with_timeout(mut self, timeout: i32) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_id(mut self, id: u32) -> Self {
        self.id = id;
        self
    }

    pub async fn send(self) -> anyhow::Result<u32> {
        self.proxy
            .notify(
                "moxapi",
                self.id,
                "",
                self.summary,
                self.body,
                Box::new([]),
                HashMap::new(),
                self.timeout,
            )
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
    }
}
