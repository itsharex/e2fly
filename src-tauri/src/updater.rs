use tauri::{AppHandle, Runtime, Url};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::UpdaterExt;

use crate::{app::before_exit_app, conf};

pub fn check_update<R: Runtime>(app: &AppHandle<R>) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        update(handle).await.expect("update failed");
    });
}

async fn update<R: Runtime>(app: AppHandle<R>) -> Result<(), tauri_plugin_updater::Error> {
    println!("start check update");

    let conf = conf::read();
    let mut updater_builder = app.updater_builder();

    if conf.active.enabled && conf.v2_fly.http.enabled {
        let http_proxy = format!(
            "http://{}:{}",
            conf.v2_fly.http.address, conf.v2_fly.http.port
        );

        let url = Url::parse(http_proxy.as_str()).expect("parse proxy url");

        println!("set proxy {}", url);

        updater_builder = updater_builder.proxy(url);
    }

    let updater = updater_builder.build()?;

    if let Some(update) = updater.check().await? {
        let mut downloaded = 0;

        println!("start download");

        let binary = update
            .download(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    println!("downloaded {downloaded} from {content_length:?}");
                },
                || {
                    println!("download finished");
                },
            )
            .await?;

        let answer = app
            .dialog()
            .message("Updates is ready, confirm to install")
            .title("E2Fly")
            .buttons(MessageDialogButtons::OkCancel)
            .blocking_show();

        if answer {
            before_exit_app(&app);
            update.install(binary)?;
        }
    } else {
        app.notification()
            .builder()
            .title("E2Fly")
            .body("You are using the latest version")
            .show()
            .unwrap();
    }

    Ok(())
}