use atspi::{
    connection::AccessibilityConnection,
    events::{
        document::DocumentEvents, focus::FocusEvents, keyboard::KeyboardEvents, mouse::MouseEvents,
        object::ObjectEvents, terminal::TerminalEvents, window::WindowEvents, AddAccessibleEvent,
        EventListenerDeregisteredEvent, EventListenerRegisteredEvent, LegacyAddAccessibleEvent,
        RemoveAccessibleEvent,
    },
    proxy::{accessible::AccessibleProxy, application::ApplicationProxy},
};
use tokio_stream::StreamExt;
use zbus::{self, zvariant::Signature, MessageType, ProxyBuilder};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const APPLICATION_INTERFACE: &str = "org.a11y.atspi.Application";
const ACCESSIBLE_INTERFACE: &str = "org.a11y.atspi.Accessible";
const ACCESSIBLE_ROOT_PATH: &str = "/org/a11y/atspi/accessible/root";

async fn atspi_setup_connection() -> Result<AccessibilityConnection> {
    // Get a connection to the AT-SPI D-Bus service
    let atspi: AccessibilityConnection = AccessibilityConnection::open().await?;

    // Register for events with `registryd` & set match rules at the a11y bus

    atspi.register_event::<MouseEvents>().await?;
    atspi.register_event::<KeyboardEvents>().await?;
    atspi.register_event::<FocusEvents>().await?;
    atspi.register_event::<WindowEvents>().await?;
    atspi.register_event::<DocumentEvents>().await?;
    atspi.register_event::<ObjectEvents>().await?;
    atspi.register_event::<TerminalEvents>().await?;

    atspi.register_event::<AddAccessibleEvent>().await?;
    atspi.register_event::<LegacyAddAccessibleEvent>().await?;
    atspi.register_event::<RemoveAccessibleEvent>().await?;

    // Regustryd events
    atspi
        .register_event::<EventListenerDeregisteredEvent>()
        .await?;
    atspi
        .register_event::<EventListenerRegisteredEvent>()
        .await?;

    Ok(atspi)
}

// Check whether the has balanced parentheses.
fn has_balanced_parentheses(signature_str: &str) -> bool {
    signature_str.chars().fold(0, |count, ch| match ch {
        '(' => count + 1,
        ')' if count != 0 => count - 1,
        _ => count,
    }) == 0
}

// Determines whether the signature has outer parentheses.
// For a signature to have outer parentheses, it must have:
// 1. Parentheses at the beginning and end of the signature.
// 2. The substring in between outer parentheses has to have balanced parentheses.
// eg. consider "(so)(av)" does not have outer parentheses, but "(so(av))" does.
fn has_outer_parentheses(sig_str: &str) -> bool {
    if let Some(subslice) = sig_str.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
        has_balanced_parentheses(subslice)
    } else {
        false
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let atspi = atspi_setup_connection().await?;
    let conn = atspi.connection();

    let mut unmarshalled_parens_signals = zbus::MessageStream::from(conn).filter(|msg| {
        msg.is_ok()
            && msg.as_ref().unwrap().message_type() == MessageType::Signal
            && has_outer_parentheses(msg.as_ref().unwrap().body_signature().unwrap().as_str())
    });

    while let Some(msg) = unmarshalled_parens_signals.next().await {
        match msg {
            Ok(msg) => {
                println!();
                println!("{}", "=".repeat(60));
                println!("      D-Bus message with unmarshalled body signature:");
                println!("{}", "=".repeat(60));

                // If the signature field is omitted, the signature is assumed to be empty.
                let signature = msg
                    .body_signature()
                    .unwrap_or(Signature::from_str_unchecked(""));

                println!(" Signature: \"{}\",", signature.as_str());

                // check if the message is an AT-SPI event
                let interface = msg
                    .interface()
                    .expect("Messages are expected to have an interface.");
                let interface_str = interface.as_str();
                if interface_str.starts_with("org.a11y") && msg.header().is_ok() {
                    let header = msg.header().unwrap();

                    let sender = header.sender().expect("No sender");
                    let sender_str = sender.unwrap().as_str();

                    let path = msg.path().unwrap();
                    let path_str = path.as_str();

                    println!(" Sender: \"{sender_str}\", Path: \"{path_str}\"");

                    let accessible_proxy: AccessibleProxy = ProxyBuilder::new(conn)
                        .interface(ACCESSIBLE_INTERFACE)?
                        .path(ACCESSIBLE_ROOT_PATH)?
                        .destination(sender_str)?
                        .build()
                        .await?;

                    let app_name = accessible_proxy
                        .name()
                        .await
                        .unwrap_or("Failed to retrieve bus application name.".to_string());

                    let role_str = accessible_proxy
                        .get_role_name()
                        .await
                        .unwrap_or("Failed to retrieve role name".to_string());

                    if let Ok(app_accessible) = accessible_proxy.get_application().await {
                        let atspi::Accessible { name, path: _ } = app_accessible;

                        let application_proxy: ApplicationProxy = ProxyBuilder::new(conn)
                            .interface(APPLICATION_INTERFACE)?
                            .path(ACCESSIBLE_ROOT_PATH)?
                            .destination(name)?
                            .build()
                            .await?;

                        let toolkit_name = application_proxy
                            .toolkit_name()
                            .await
                            .unwrap_or("Failed to retrieve toolkit name".to_string());

                        println!("  Toolkit name: {toolkit_name}");
                    }

                    println!("  Application name: \"{app_name}\"");
                    println!("  Object role: \"{role_str}\"");
                }
            }

            _ => {
                println!("I do not expect to be printed! This is a bug.");
            }
        }
    }

    Ok(())
}
