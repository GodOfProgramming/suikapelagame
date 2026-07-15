use super::AppEditableText;
use crate::{
	GameState,
	game::{ApClient, SessionData},
};
use bevy::{
	color::palettes::tailwind::SLATE_500, ecs::system::SystemParam, prelude::*, text::EditableText,
	ui::FlexDirection,
};
use bevy_defer::{AsyncCommandsExtension, AsyncWorld};

#[derive(SceneComponent, Default, Clone)]
pub struct LoginFormComponent;

impl LoginFormComponent {
	fn scene() -> impl Scene {
		bsn! {
			Node {
				padding: UiRect::all(px(16)),
				border_radius: BorderRadius::all(px(16))
				max_width: px(320),
				display: Display::Flex,
				flex_direction: FlexDirection::Column,
			}
			BackgroundColor({ Color::srgb_u8(0xff, 0xea, 0xce) })
			Children [
				login_form()
			]
		}
	}
}

fn login_form() -> impl Scene {
	bsn! {
		Node {
			display: Display::Flex,
			flex_direction: FlexDirection::Column,
			justify_content: JustifyContent::End,
		}
		Children [
			host_input(),
			port_input(),
			name_input(),
			password_input(),
			login_button(),
		]
	}
}

#[derive(SystemParam)]
struct FormData<'w, 's> {
	host: Single<'w, 's, &'static EditableText, With<HostInput>>,
	port: Single<'w, 's, &'static EditableText, With<PortInput>>,
	name: Single<'w, 's, &'static EditableText, With<NameInput>>,
	pass: Single<'w, 's, &'static EditableText, With<PasswordInput>>,
}

const DEFAULT_HOST: &str = if cfg!(debug_assertions) {
	"0.0.0.0"
} else {
	"archipelago.gg"
};

const DEFAULT_PORT: &str = if cfg!(debug_assertions) { "38281" } else { "" };

#[derive(Component, Default, Clone)]
#[require(EditableText::new("archipelago.gg"))]
struct HostInput;

fn host_input() -> impl Scene {
	bsn! {
		column()
		Children [
			label_text("Host"),
			HostInput @AppEditableText {
				@name: "Host Input",
				@placeholder_text: DEFAULT_HOST,
			}
		]
	}
}

#[derive(Component, Default, Clone)]
struct PortInput;

fn port_input() -> impl Scene {
	bsn! {
		column()
		Children [
			label_text("Port"),
			PortInput @AppEditableText {
				@name: "Port Input",
				@placeholder_text: DEFAULT_PORT
			}
		]
	}
}

#[derive(Component, Default, Clone)]
struct NameInput;

fn name_input() -> impl Scene {
	bsn! {
		column()
		Children [
			label_text("Name"),
			NameInput @AppEditableText {
				@name: "Name Input"
			}
		]
	}
}

#[derive(Component, Default, Clone)]
#[require(EditableText)]
struct PasswordInput;

fn password_input() -> impl Scene {
	bsn! {
		column()
		Children [
			label_text("Password"),
			PasswordInput @AppEditableText {
				@name: "Password Input"
			}
		]
	}
}

fn login_button() -> impl Scene {
	bsn! {
		Button
		BackgroundColor({SLATE_500})
		Node {
			justify_content: JustifyContent::Center
		}
		Children [
			Text::new("Login") TextColor::BLACK
		]
		on(on_login_click)
	}
}

fn on_login_click(_: On<Pointer<Click>>, mut commands: Commands, data: FormData) -> Result {
	let host = data.host.value().to_string();
	let port = data.port.value().to_string();
	let name = data.name.value().to_string();
	let pass = data.pass.value().to_string();

	if host.is_empty() {
		return Err(BevyError::error("host cannot be empty"));
	}

	if port.is_empty() {
		return Err(BevyError::error("port cannot be empty"));
	}

	if port.parse::<u16>().is_err() {
		return Err(BevyError::error("port is not valid"));
	}

	if name.is_empty() {
		return Err(BevyError::error("name cannot be empty"));
	}

	let mut opts = archipelago_rs::ConnectionOptions::new();

	if !pass.is_empty() {
		opts = opts.password(pass);
	}

	let url = format!("{host}:{port}");
	commands.spawn_task(move || async {
		info!("Starting login...");

		let login_result =
			archipelago_rs::Client::<SessionData>::connect(url, name, Some("Suikapelagame"), opts).await;

		match login_result {
			Ok(client) => {
				info!(
					session_data = format!("{:?}", client.slot_data()),
					"Login complete"
				);

				AsyncWorld.insert_resource(ApClient::new(client));
				AsyncWorld.set_state(GameState::InGame)?;

				Ok(())
			}
			Err(err) => {
				error!(err = err.to_string(), "Failed to login to archipelago");
				Ok(())
			}
		}
	});

	Ok(())
}

fn column() -> impl Scene {
	bsn! {
		Node {
			display: Display::Flex,
			flex_direction: FlexDirection::Column,
		}
	}
}

fn label_text(txt: impl Into<String>) -> impl Scene {
	let txt = Text::new(txt);
	bsn! {
		Label
		TextColor::BLACK
		template_value(txt)
	}
}
