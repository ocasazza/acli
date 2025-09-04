use nix_rust_template::Post;
use web_sys::window;

/// The main function of the client application.
///
/// This function sets up the panic hook, creates a new `Post`,
/// and then appends a greeting to the document body.
fn main() {
    console_error_panic_hook::set_once();
    let post = Post {
        title: "Nix Rust Template".to_string(),
        author_name: "Olive Casazza".to_string(),
        text: "🚀 Blazingly Fast".to_string(),
    };
    let document = window()
        .and_then(|win| win.document())
        .expect("Could not access the document");
    let body = document.body().expect("Could not access document.body");
    let greeting = format!("Hello {}!", post.author_name);
    let text_node = document.create_text_node(&greeting);
    body.append_child(text_node.as_ref())
        .expect("Failed to append text");
}

// uncomment the following, and the additional dependencies
// in Cargo.toml for a more complex yew based frontend

// use gloo::net;
// use yew::prelude::*;
// use nix_rust_template::Post;

// async fn request_posts() -> Vec<Post> {
//     net::http::Request::get("http://localhost:8000/posts")
//         .send()
//         .await
//         .expect("Failed to connect with server, is it runnig at localhost:8000?")
//         .json()
//         .await
//         .expect("Received invalid response from server")
// }

// struct AppStruct(Vec<Post>);

// impl Component for AppStruct {
//     type Message = Vec<Post>;
//     type Properties = ();

//     fn create(ctx: &Context<Self>) -> Self {
//         ctx.link().send_future(request_posts());
//         Self(Default::default())
//     }

//     fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
//         self.0 = msg;
//         true
//     }

//     fn view(&self, _ctx: &Context<Self>) -> Html {
//         html! {
//           <ul>
//             {for self.0.iter().map(|post| html! {
//               <li>
//                 {format!("{} by {}", post.title, post.author_name)}
//               </li>
//             })}
//           </ul>
//         }
//     }
// }

// fn main() {
//     yew::Renderer::<AppStruct>::new().render();
// }
