use std::collections::HashMap;
use std::vec;

use gloo::file::callbacks::FileReader;
use gloo::file::File;
use rand::Rng;
use wasm_bindgen::JsValue;
use web_sys::{DragEvent, FileList};
use yew::prelude::*;
use yew::{html, Callback, Component, Context, Html};

mod diff;

use diff::{get_diffs, Diff};

struct FileDetails {
    name: String,
    file_type: String,
    data: Vec<u8>,
}

pub enum WhichFile {
    File1,
    File2,
}

pub enum Msg {
    Loaded(String, String, Vec<u8>, WhichFile),
    File(File, WhichFile),
    ButtonClick(WhichFile),
}

#[derive(Default)]
pub struct App {
    readers: HashMap<String, FileReader>,
    file1: Option<FileDetails>,
    file2: Option<FileDetails>,
    diffs1: Vec<Diff>,
    diffs2: Vec<Diff>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::ButtonClick(WhichFile::File1));
        ctx.link().send_message(Msg::ButtonClick(WhichFile::File2));
        Self::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded(file_name, file_type, data, which_file) => {
                *(match which_file {
                    WhichFile::File1 => &mut self.file1,
                    WhichFile::File2 => &mut self.file2,
                }) = Some(FileDetails {
                    data,
                    file_type,
                    name: file_name.clone(),
                });
                self.readers.remove(&file_name);

                self.update_diffs();

                true
            }
            Msg::File(file, which_file) => {
                let file_name = file.name();
                let file_type = file.raw_mime_type();

                let task = {
                    let link = ctx.link().clone();
                    let file_name = file_name.clone();

                    gloo::file::callbacks::read_as_bytes(&file, move |res| {
                        link.send_message(Msg::Loaded(
                            file_name,
                            file_type,
                            res.expect("failed to read file"),
                            which_file,
                        ))
                    })
                };
                self.readers.insert(file_name, task);
                true
            }
            Msg::ButtonClick(which_file) => {
                let mut rng = rand::thread_rng();
                let data: Vec<u8> = (0..1000).map(|_| rng.gen_range(0..=255)).collect();
                *(match which_file {
                    WhichFile::File1 => &mut self.file1,
                    WhichFile::File2 => &mut self.file2,
                }) = Some(FileDetails {
                    name: "random".to_string(),
                    file_type: "test".to_string(),
                    data,
                });
                self.update_diffs();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let data1 = if let Some(file) = &self.file1 {
            file.data.clone()
        } else {
            Vec::new()
        };
        let data2 = if let Some(file) = &self.file2 {
            file.data.clone()
        } else {
            Vec::new()
        };

        fn prevent_default(event: DragEvent) {
            event.prevent_default();
        }

        let load_file1 = |event: DragEvent| {
            event.prevent_default();
            let files = event.data_transfer().unwrap().files();
            Self::load_file(files, WhichFile::File1)
        };

        let load_file2 = |event: DragEvent| {
            event.prevent_default();
            let files = event.data_transfer().unwrap().files();
            Self::load_file(files, WhichFile::File2)
        };

        let file1_name = if let Some(file) = &self.file1 {
            &file.name
        } else {
            "no file"
        };

        let file2_name = if let Some(file) = &self.file2 {
            &file.name
        } else {
            "no file"
        };

        html! {
            <>
                <h1>{ "hex diff test" }</h1>
                <div class="hex-container">
                    <div>
                        <button onclick={ctx.link().callback(|_| Msg::ButtonClick(WhichFile::File1))}>
                            { "randomize" }
                        </button>
                        <text>{file1_name}</text>
                        <div class="drop-container"
                            ondrop={ctx.link().callback(load_file1)}
                            ondragover={Callback::from(prevent_default)}
                            ondragenter={Callback::from(prevent_default)}
                            >
                            <DrawAddressColumn />
                            <DrawHexGrid data={data1.clone()} diffs={self.diffs1.clone()} />
                            <DrawAsciiGrid data={data1.clone()} diffs={self.diffs1.clone()} />
                        </div>
                    </div>
                    <div>
                        <button onclick={ctx.link().callback(|_| Msg::ButtonClick(WhichFile::File2))}>
                            { "randomize" }
                        </button>
                        <text>{file2_name}</text>
                        <div class="drop-container"
                            ondrop={ctx.link().callback(load_file2)}
                            ondragover={Callback::from(prevent_default)}
                            ondragenter={Callback::from(prevent_default)}
                            >
                            <DrawAddressColumn />
                            <DrawHexGrid data={data2.clone()} diffs={self.diffs2.clone()} />
                            <DrawAsciiGrid data={data2.clone()} diffs={self.diffs2.clone()} />
                        </div>
                    </div>
                </div>
            </>
        }
    }
}

impl App {
    fn load_file(files: Option<FileList>, which_file: WhichFile) -> Msg {
        let files = files.unwrap();
        let file = js_sys::try_iter(&files)
            .unwrap()
            .unwrap()
            .map(|v| web_sys::File::from(v.unwrap()))
            .map(File::from)
            .next()
            .unwrap();

        Msg::File(file, which_file)
    }

    fn update_diffs(&mut self) {
        let (diffs1, diffs2) = if let (Some(file1), Some(file2)) = (&self.file1, &self.file2) {
            get_diffs(&file1.data, &file2.data)
        } else {
            (vec![], vec![])
        };
        self.diffs1 = diffs1;
        self.diffs2 = diffs2;
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

#[derive(Properties, PartialEq, Debug)]
struct DataProps {
    data: Vec<u8>,
    diffs: Vec<Diff>,
}

#[function_component]
fn DrawHexGrid(data: &DataProps) -> Html {
    html! {
        <div class="hex-grid">
            { for data.data.iter()
                .zip(data.diffs.iter())
                .map(|(x,y)| html! { <HexCell data={*x} diff={*y} /> }) }
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct HexCellProps {
    data: u8,
    diff: Diff,
}

#[function_component]
fn HexCell(props: &HexCellProps) -> Html {
    let color = match props.diff {
        Diff::Same => "white",
        Diff::Different => "red",
        Diff::NoOther => "gray",
    };
    html! {
        <div class="hex-cell" style={format!("background-color: {}", color)}>
            { format!("{:02X}", props.data) }
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct AddressCellProps {
    data: u8,
}

#[function_component]
fn AddressCell(props: &AddressCellProps) -> Html {
    html! {
        <div class="address-cell" style="background-color: gray">
            { format!("{:08X}", props.data as u64*16) }
        </div>
    }
}

#[function_component]
fn DrawAddressColumn() -> Html {
    html! {
        <div class="address-column">
            { for (0..100).map(|x| html! { <AddressCell data={x} /> }) }
        </div>
    }
}

#[function_component]
fn DrawAsciiGrid(data: &DataProps) -> Html {
    html! {
        <div class="ascii-grid">
        { for data.data.iter().map(|x| html! { <AsciiCell data={*x} /> }) }
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct AsciiCellProps {
    data: u8,
}

#[function_component]
fn AsciiCell(props: &AsciiCellProps) -> Html {
    html! {
        <div class="hex-cell" style="background-color: gray">
            { format!("{}", props.data as char) }
        </div>
    }
}
