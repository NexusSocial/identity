use leptos::{
	component,
	html::ElementChild as _,
	mount::mount_to_body,
	prelude::{OnAttribute as _, Set as _},
	reactive::signal::signal,
	view, IntoView,
};

fn main() {
	console_error_panic_hook::set_once();
	tracing_wasm::set_as_global_default();
	tracing::debug!("started wasm");
	mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
	let (count, set_count) = signal(0);
	view! {
		<h1>"Hello World"</h1>
		<button on:click=move |_| set_count.set(3)>
		  "Click me: "{count}
		</button>
	}
}
