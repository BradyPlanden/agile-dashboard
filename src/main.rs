fn main() {
    let root = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("app"))
        .expect("index.html contains an #app element");
    yew::Renderer::<agile_dashboard::App>::with_root(root).render();
}
