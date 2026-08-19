use std::marker::PhantomData;

use super::AppEditableText;
use crate::{component_name, game::archipelago::ApClient, ui::components::AppButton};
use bevy::{ecs::system::SystemParam, prelude::*, text::EditableText, ui::FlexDirection};
use bevy_defer::AsyncCommandsExtension;

#[derive(SceneComponent, Default, Clone)]
pub struct LoginFormComponent;

impl LoginFormComponent {
	fn scene() -> impl Scene {
		bsn! {
			{ component_name!(LoginFormComponent) }
			Node {
				padding: UiRect::all(px(16)),
				border_radius: BorderRadius::all(px(16))
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
			@FormInput<HostInput> {
				@label: "Host",
				@placeholder: DEFAULT_HOST,
			},
			@FormInput<PortInput> {
				@label: "Port",
				@placeholder: DEFAULT_PORT,
			},
			@FormInput<NameInput> {
				@label: "Name",
			},
			@FormInput<PasswordInput> {
				@label: "Password",
			},
			@AppButton
			Node { justify_content: JustifyContent::Center }
			Children [ Text::new("Login") TextColor::BLACK ]
			on(on_login_click)
		]
	}
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

#[derive(Component, Default, Clone)]
struct PortInput;

#[derive(Component, Default, Clone)]
struct NameInput;

#[derive(Component, Default, Clone)]
#[require(EditableText)]
struct PasswordInput;

#[derive(SceneComponent, Default, Clone)]
#[scene(FormInputProps<C>)]
struct FormInput<C: Component + Default + Clone + Unpin>(PhantomData<C>);

#[derive(Default)]
struct FormInputProps<C> {
	component: C,
	label: String,
	placeholder: String,
}

impl<C> FormInput<C>
where
	C: Component + Default + Clone + Unpin,
{
	fn scene(props: FormInputProps<C>) -> impl Scene {
		super::column(bsn_list! [
			Label TextColor::BLACK Text({props.label.clone()}),
			template_value(props.component)
			@AppEditableText {
				@name: {props.label},
				@placeholder_text: {props.placeholder},
			}
		])
	}
}

#[derive(SystemParam)]
struct FormData<'w, 's> {
	host: Single<'w, 's, &'static EditableText, With<HostInput>>,
	port: Single<'w, 's, &'static EditableText, With<PortInput>>,
	name: Single<'w, 's, &'static EditableText, With<NameInput>>,
	pass: Single<'w, 's, &'static EditableText, With<PasswordInput>>,
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

	let url = format!("{host}:{port}");
	commands.spawn_task(|| async {
		ApClient::login(url, name, if pass.is_empty() { None } else { Some(pass) }).await;
		Ok(())
	});

	Ok(())
}
