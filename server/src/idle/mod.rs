use zbus::Connection;

#[zbus::proxy(
    name = "org.freedesktop.ScreenSaver",
    default_service = "org.freedesktop.ScreenSaver",
    default_path = "/org/freedesktop/ScreenSaver"
)]
trait ScreenSaver {
    async fn simulate_user_activity(&self) -> zbus::Result<()>;

    async fn inhibit(&self, application_name: &str, reason_for_inhibit: &str) -> zbus::Result<u32>;

    async fn un_inhibit(&self, cookie: u32) -> zbus::Result<()>;
}

#[zbus::proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
trait LoginManager {
    async fn lock_session(&self, session_id: &str) -> zbus::Result<()>;

    async fn unlock_session(&self, session_id: &str) -> zbus::Result<()>;
}

pub struct Idle {
    cookie: Option<u32>,
    system_conn: Connection,
    session_conn: Connection,
    screen_saver: ScreenSaverProxy<'static>,
    login_manager: LoginManagerProxy<'static>,
}

impl Idle {
    pub async fn new() -> anyhow::Result<Self> {
        let system_conn = zbus::Connection::system().await?;
        let session_conn = zbus::Connection::session().await?;

        let login_manager = LoginManagerProxy::new(&system_conn).await?;
        let screen_saver = ScreenSaverProxy::new(&session_conn).await?;

        Ok(Self {
            cookie: None,
            login_manager,
            screen_saver,
            session_conn,
            system_conn,
        })
    }

    pub async fn lock(&self) -> anyhow::Result<()> {
        self.login_manager.lock_session("auto").await?;

        Ok(())
    }

    pub async fn unlock(&self) -> anyhow::Result<()> {
        self.login_manager.unlock_session("auto").await?;

        Ok(())
    }

    pub async fn simulate_user_activity(&self) -> anyhow::Result<()> {
        self.screen_saver.simulate_user_activity().await?;

        Ok(())
    }

    pub async fn inhibit(&mut self) -> anyhow::Result<()> {
        match self.cookie.is_some() {
            true => return anyhow::Result::Err(anyhow::anyhow!("Already inhibited")),
            false => self.cookie = self.screen_saver.inhibit("", "").await.ok(),
        }

        Ok(())
    }

    pub async fn uninhibit(&mut self) -> anyhow::Result<()> {
        match self.cookie.take() {
            Some(cookie) => self.screen_saver.un_inhibit(cookie).await?,
            None => return anyhow::Result::Err(anyhow::anyhow!("Not inhibited")),
        }

        Ok(())
    }
}
