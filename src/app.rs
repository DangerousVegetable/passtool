use native_windows_gui as nwg;
use native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::{NativeUi, WindowFlags};
use passtool::PassTable;

use std::{cell::RefCell, thread, time::Duration, path::Path};
use winapi::um::winuser::{GetAsyncKeyState, VK_CONTROL, VK_MENU};
//const flaggg: WindowFlags = WindowFlags::POPUP;

pub const SAVEFILE: &str = "passwords.pt";

#[derive(Default, NwgUi)]
pub struct PassToolApp {
    
    passtable: PassTable,

    #[nwg_resource(source_file: Some("./resources/icon.ico"))]
    icon: nwg::Icon, // icon

    #[nwg_control(flags:"SYS_MENU", icon: Some(&data.icon))]
    #[nwg_events( OnWindowClose: [PassToolApp::exit], OnInit: [PassToolApp::load_data])]
    window: nwg::Window, // hidden window

    #[nwg_control(parent: Some(&data.window), size: (500, 250), title: "PassTool Overlay", topmost: true, center: true, flags:"POPUP", icon: Some(&data.icon))]
    popup_window: nwg::Window, // main popup window

    #[nwg_control(spacing: 10)]
    #[nwg_layout(parent: popup_window)]
    layout: nwg::GridLayout, // layout

    #[nwg_control(item_count: 10, list_style: nwg::ListViewStyle::Detailed, focus: true,
        ex_flags: nwg::ListViewExFlags::GRID | nwg::ListViewExFlags::FULL_ROW_SELECT, 
        background_color: [100,100,100]
        )]
    #[nwg_layout_item(layout: layout, col: 0, col_span: 4, row: 0, row_span: 7)]
    #[nwg_events(OnListViewItemActivated: [PassToolApp::open_input], OnListViewItemChanged: [PassToolApp::disable_input])]
    data_view: nwg::ListView, // list of passwords

    #[nwg_control(parent: popup_window, text: "", placeholder_text: Some("Choose password"), password: Some('*'), flags: "VISIBLE")]
    #[nwg_events(OnKeyEnter: [])]
    #[nwg_layout_item(layout: layout, col: 0, col_span: 4, row: 7)]
    input_box: nwg::TextInput, // input 
    input_text: RefCell<String>, // shared string

    #[nwg_control(icon: Some(&data.icon), tip: Some("PassTool"))]
    #[nwg_events(MousePressLeftUp: [PassToolApp::show_tray_menu], OnContextMenu: [PassToolApp::show_tray_menu])]
    tray: nwg::TrayNotification, // tray notification

    #[nwg_control(parent: window, popup: true)]
    tray_menu: nwg::Menu, // tray menu

    #[nwg_control(parent: tray_menu, text: "Toggle Overlay \t [Ctrl + Alt + P]")]
    #[nwg_events(OnMenuItemSelected: [PassToolApp::toggle_overlay])]
    tray_item1: nwg::MenuItem, // tray menu option

    #[nwg_control(parent: tray_menu, text: "Exit")]
    #[nwg_events(OnMenuItemSelected: [PassToolApp::exit])]
    tray_item2: nwg::MenuItem, // tray menu option

    #[nwg_control(parent: window)]
    #[nwg_events(OnNotice: [PassToolApp::toggle_overlay])]
    shortcut_notice: nwg::Notice // being sent when shortcut is pressed
}

impl PassToolApp {
    fn show_tray_menu(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.tray_menu.popup(x, y);
    }

    fn toggle_overlay(&self) {
        use winapi::um::winuser::{ShowWindow, IsWindowVisible, SW_HIDE, SW_SHOW};
        unsafe{
            let hWnd = self.popup_window.handle.hwnd().unwrap();
            if IsWindowVisible(hWnd) == 0 {
                let (mut x, mut y) = nwg::GlobalCursor::position();
                let (w, h) = self.popup_window.size();
                let (w, h) = (w as i32, h as i32);
                let [total_width, total_height] = [nwg::Monitor::width(), nwg::Monitor::height()];
                
                x = std::cmp::min(total_width-w, x);
                y = std::cmp::min(total_height-h-50, y);

                self.popup_window.set_position(x, y);
                ShowWindow(hWnd, SW_SHOW); // show the window for the new style to come into effect
            }
            else { 
                ShowWindow(hWnd, SW_HIDE); 
            }
        }
    }

    fn disable_input(&self) {
        self.input_box.set_text("");
        self.input_box.set_placeholder_text(Some("Choose password"));
        self.input_box.set_enabled(false);
        self.input_box.set_visible(true);
    }

    fn open_input(&self) {
        let ind = self.data_view.selected_item().unwrap();
        //self.popup_window.set_size(500, 350);
        self.input_box.set_enabled(true);
        *self.input_text.borrow_mut() = format!("Input key for the password \"{}\":", ind);
        self.input_box.set_placeholder_text(Some(self.input_text.borrow_mut().as_str()));
        self.input_box.set_visible(true);
    }

    fn tray_notification(&self) {
        let flags = nwg::TrayNotificationFlags::USER_ICON | nwg::TrayNotificationFlags::LARGE_ICON;
        self.tray.show("PassTool working in tray", None, Some(flags), Some(&self.icon));
    }

    fn expect_shortcut(&self) {
        let sender = self.shortcut_notice.sender();
        thread::spawn(move || {
            let mut state = false;
            loop {
                unsafe {
                    let new_state = GetAsyncKeyState(VK_CONTROL) as i32 & 0x8000 != 0 && 
                                            GetAsyncKeyState(VK_MENU) as i32 & 0x8000 != 0 &&
                                            GetAsyncKeyState('P' as i32) as i32 & 0x8000 != 0;
                    if new_state && !state {
                        sender.notice();
                    }
                    state = new_state;
                    thread::sleep(Duration::new(0, 10000000));
                }
            }
        });
    }

    fn load_data(&self) {
        use winapi::um::winuser::{SetWindowLongPtrA, GWL_STYLE, GetWindowLongA, WS_EX_TOOLWINDOW, WS_EX_APPWINDOW, ShowWindow, SW_HIDE, SW_SHOW};
        self.input_box.set_visible(true);
        self.input_box.set_enabled(false);
        unsafe{
            let hWnd = self.popup_window.handle.hwnd().unwrap();
            let mut style = GetWindowLongA(hWnd, GWL_STYLE) as u32;
            style |= WS_EX_TOOLWINDOW;   // flags don't work - windows remains in taskbar
            style &= !(WS_EX_APPWINDOW);
            ShowWindow(hWnd, SW_HIDE); // hide the window
            SetWindowLongPtrA(hWnd, GWL_STYLE, style as isize); // set the style
            
        }
        self.tray_notification();
        self.expect_shortcut();
        
        let dv = &self.data_view;

        dv.insert_column("Name");
        dv.insert_column("Description");
        dv.insert_column("Affiliated apps");
        dv.set_headers_enabled(true);
        let mut names: Vec<&String> = self.passtable.get_names().collect();
        names.sort();
        self.update_list(&names);
    }

    fn update_list(&self, names: &[&String]) {
        let dv = &self.data_view;
        for name in names {
            let meta = self.passtable.get_metadata(name).unwrap();
            let ind: i32 = dv.len() as i32;
            dv.insert_item(nwg::InsertListViewItem {
                index: Some(ind),
                column_index: 0,
                text: Some(name.to_string()),
                image: None,
            });
            dv.insert_item(nwg::InsertListViewItem {
                index: Some(ind),
                column_index: 1,
                text: Some(meta.description.clone()),
                image: None,
            });
            dv.insert_item(nwg::InsertListViewItem {
                index: Some(ind),
                column_index: 2,
                text: Some(format!("{:?}", meta.apps)),
                image: None,
            });
        }
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path.push(SAVEFILE);
        self.passtable.to_file(&path)
    }

    fn exit(&self) {
        let _ = self.save();
        nwg::stop_thread_dispatch();
    }

}

pub fn run() {
    let pt = PassTable::default();
    
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path.push(SAVEFILE);
    
    if !path.exists() {
        pt.to_file(&path).unwrap();
    }
    let pt = PassTable::from_file(&path).unwrap();
    
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let _app = PassToolApp::build_ui(PassToolApp{passtable: pt, ..Default::default()}).expect("Failed to build UI");

    nwg::dispatch_thread_events();
}