use native_windows_gui as nwg;
use native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::{NativeUi, WindowFlags};
use passtool::{generator, PassTable, Password, PasswordMeta};

use std::{cell::RefCell, thread, time::Duration, path::Path};
use winapi::{shared::{minwindef::{HMODULE, MAX_PATH}, ntdef::{LPCWSTR, WCHAR}, windef::POINT}, um::{uxtheme::SetWindowTheme, winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ}, winuser::{GetAsyncKeyState, VK_CONTROL, VK_MENU}}};
//const flaggg: WindowFlags = WindowFlags::POPUP;

pub const SAVEFILE: &str = "passwords.pt";

#[derive(Default, NwgUi)]
pub struct PassToolApp {
    passtable: RefCell<PassTable>,

    #[nwg_resource(source_file: Some("./resources/icon.ico"))]
    icon: nwg::Icon, // icon

    #[nwg_control(flags:"SYS_MENU", icon: Some(&data.icon))]
    #[nwg_events( OnWindowClose: [PassToolApp::exit], OnInit: [PassToolApp::load_data])]
    window: nwg::Window, // hidden window

    #[nwg_control(parent: Some(&data.window), size: (500, 350), title: "PassTool Overlay", topmost: true, center: true, flags: "WINDOW|POPUP", icon: Some(&data.icon))]
    popup_window: nwg::Window, // main popup window

    #[nwg_control(spacing: 15)]
    #[nwg_layout(parent: popup_window)]
    layout: nwg::GridLayout, // layout

    #[nwg_control(text: "App-specific passwords:", flags: "VISIBLE")]
    #[nwg_layout_item(layout: layout, row: 0, col_span: 4)]
    label1: nwg::Label,
    
    #[nwg_control(item_count: 10, list_style: nwg::ListViewStyle::Detailed, focus: true,
        ex_flags: nwg::ListViewExFlags::GRID | nwg::ListViewExFlags::FULL_ROW_SELECT, 
        background_color: [100,100,100]
    )]
    #[nwg_layout_item(layout: layout, col_span: 4, row: 1, row_span: 3)]
    #[nwg_events(OnListViewItemActivated: [PassToolApp::enable_input], OnListViewItemChanged: [PassToolApp::disable_input])]
    rec_pass_view: nwg::ListView, // list of passwords
    rec_pass_names: RefCell<Vec<String>>,
    active_process: RefCell<String>,
    
    #[nwg_control(text: "All passwords:", flags: "VISIBLE")]
    #[nwg_layout_item(layout: layout, row: 4, col_span: 4)]
    label2: nwg::Label,
    #[nwg_control(item_count: 10, list_style: nwg::ListViewStyle::Detailed, focus: true,
        ex_flags: nwg::ListViewExFlags::GRID | nwg::ListViewExFlags::FULL_ROW_SELECT, 
        background_color: [100,100,100]
        )]
    #[nwg_layout_item(layout: layout, col_span: 4, row: 5, row_span: 7)]
    #[nwg_events(OnListViewItemActivated: [PassToolApp::enable_input], OnListViewItemChanged: [PassToolApp::disable_input])]
    pass_view: nwg::ListView, // list of passwords
    pass_names: RefCell<Vec<String>>,

    #[nwg_control(parent: popup_window, text: "", placeholder_text: Some("Choose password"), password: Some('*'), flags: "VISIBLE")]
    #[nwg_events(OnKeyEnter: [])]
    #[nwg_layout_item(layout: layout, col_span: 4, row: 12)]
    input_box: nwg::TextInput, // input 
    input_text: RefCell<String>, // shared string

    #[nwg_control(parent: popup_window, text: "Add")]
    #[nwg_layout_item(layout: layout, row: 1, col: 4, col_span: 1, row_span: 2)]
    #[nwg_events(OnButtonClick: [PassToolApp::show_add_password])]
    new_button: nwg::Button,

    #[nwg_control(parent: Some(&data.popup_window), title: "Add password", size: (300, 250), flags: "WINDOW|DISABLED")]
    add_password_window: nwg::Window, // window for adding password
    #[nwg_control(spacing: 5)]
    #[nwg_layout(parent: popup_window)]
    add_layout: nwg::GridLayout, // layout
    #[nwg_control(parent: add_password_window, text: "Name:")]
    #[nwg_layout_item(layout: add_layout, col: 0, row: 0)]
    password_name_label: nwg::Label,
    #[nwg_control(parent: add_password_window, text: "", placeholder_text: Some("anything"), flags: "VISIBLE")]
    #[nwg_layout_item(layout: add_layout, col: 1, row: 0, col_span: 2)]
    password_name_input: nwg::TextInput,
    #[nwg_control(parent: add_password_window, text: "Password:")]
    #[nwg_layout_item(layout: add_layout, col: 0, row: 1)]
    password_label: nwg::Label,
    #[nwg_control(parent: add_password_window, text: "", placeholder_text: Some("password"), password: Some('*'), flags: "VISIBLE")]
    #[nwg_layout_item(layout: add_layout, col: 1, row: 1, col_span: 2)]
    password_input: nwg::TextInput,
    #[nwg_control(parent: add_password_window, text: "Key:")]
    #[nwg_layout_item(layout: add_layout, col: 0, row: 2)]
    password_key_label: nwg::Label,
    #[nwg_control(parent: add_password_window, text: "", placeholder_text: Some("key"), password: Some('*'), flags: "VISIBLE")]
    #[nwg_layout_item(layout: add_layout, col: 1, row: 2, col_span: 2)]
    password_key_input: nwg::TextInput,
    #[nwg_control(parent: add_password_window, text: "Repeat key:")]
    #[nwg_layout_item(layout: add_layout, col: 0, row: 3)]
    password_repeat_key_label: nwg::Label,
    #[nwg_control(parent: add_password_window, text: "", placeholder_text: Some("key again"), password: Some('*'), flags: "VISIBLE")]
    #[nwg_layout_item(layout: add_layout, col: 1, row: 3, col_span: 2)]
    password_repeat_key_input: nwg::TextInput,
    #[nwg_control(parent: add_password_window, text: "Description (optional):")]
    #[nwg_layout_item(layout: add_layout, col: 0, row: 4)]
    password_description_label: nwg::Label,
    #[nwg_control(parent: add_password_window, text: "", placeholder_text: Some("anything"), flags: "VISIBLE")]
    #[nwg_layout_item(layout: add_layout, col: 1, row: 4, col_span: 2)]
    password_description_input: nwg::TextInput,
    #[nwg_control(parent: add_password_window, text: "Add")]
    #[nwg_layout_item(layout: add_layout, col: 0, row: 5, col_span: 3)]
    #[nwg_events(OnButtonClick: [PassToolApp::add_password])]
    password_add_button: nwg::Button,

    #[nwg_control(icon: Some(&data.icon), tip: Some("PassTool"))]
    #[nwg_events(MousePressLeftUp: [PassToolApp::toggle_overlay], MousePressRightUp: [PassToolApp::show_tray_menu])]
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
    fn add_password(&self) {
        if self.password_key_input.text() != self.password_repeat_key_input.text(){
            nwg::modal_error_message(self.add_password_window.handle, "Warning!", "Keys do not match!", );
            return;
        }
        let name = self.password_name_input.text();
        if name.len() == 0 {
            nwg::modal_error_message(self.add_password_window.handle, "Warning!", "Empty name is not allowed!");
            return;
        }
        let key = self.password_key_input.text();
        if key.len() == 0 {
            nwg::modal_error_message(self.add_password_window.handle, "Warning!", "Empty key is not allowed!");
            return;
        }
        let password = self.password_input.text();
        if password.len() == 0 {
            nwg::modal_error_message(self.add_password_window.handle, "Warning!", "Empty password is not allowed!");
            return;
        }
        let description = self.password_description_input.text();
        let _ = self.passtable.borrow_mut().add_password(&name, &password, PasswordMeta::new(description, Default::default()), &key);
        let _ = self.save();
        self.clear_add_password();
        self.add_password_window.set_visible(false);
        self.update_list();
    }

    fn clear_add_password(&self) {
        self.password_name_input.set_text("");
        self.password_key_input.set_text("");
        self.password_repeat_key_input.set_text("");
        self.password_input.set_text("");
        self.password_input.set_text("");
    }

    fn show_add_password(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.add_password_window.set_position(x, y);
        self.add_password_window.set_enabled(true);
        self.add_password_window.set_visible(true);
    }

    fn show_tray_menu(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.tray_menu.popup(x, y);
    }

    fn toggle_overlay(&self) {
        use winapi::um::{psapi::GetModuleFileNameExW, processthreadsapi::OpenProcess, winuser::{ShowWindow, IsWindowVisible, SW_HIDE, SW_SHOW, WindowFromPoint, GetWindowThreadProcessId}};
        unsafe{
            if !self.popup_window.visible() {                
                let (mut x, mut y) = nwg::GlobalCursor::position(); //cursor position
                let mut process_name = [0 as WCHAR; MAX_PATH]; //process name
                let active_hwnd = WindowFromPoint(POINT{x,y}); //hndw of the window below the cursor
                let mut process_id: u32 = 0; //process id of the window below the cursor
                GetWindowThreadProcessId(active_hwnd, &mut process_id);
                let hprocess = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, process_id); //process handle of the window below the cursos
                GetModuleFileNameExW(hprocess, 0 as HMODULE, process_name.as_mut_ptr(), MAX_PATH as u32);
                *self.active_process.borrow_mut() = process_name.iter().map(|x| {char::from_u32(*x as u32).unwrap()}).collect(); //setting active_process to the name of the underlying window process
                
                let (w, h) = self.popup_window.size();
                let (w, h) = (w as i32, h as i32);
                let [total_width, total_height] = [nwg::Monitor::width(), nwg::Monitor::height()];
                
                x = std::cmp::min(total_width-w, x);
                y = std::cmp::min(total_height-h-50, y);

                self.popup_window.set_position(x, y);
                self.update_list();
                self.popup_window.set_enabled(true);
                self.popup_window.set_visible(true);
            }
            else { 
                self.add_password_window.set_visible(false);
                self.popup_window.set_visible(false); 
            }
        }
    }

    fn disable_input(&self) {
        self.input_box.set_text("");
        self.input_box.set_placeholder_text(Some("Choose password"));
        self.input_box.set_enabled(false);
        self.input_box.set_visible(true);
    }

    fn enable_input(&self) {
        let ind = self.pass_view.selected_item().unwrap();
        let pass_name = &self.pass_names.borrow()[ind];
        self.input_box.set_enabled(true);
        *self.input_text.borrow_mut() = format!("Input key for the password \"{pass_name}\":");
        self.input_box.set_placeholder_text(Some(self.input_text.borrow_mut().as_str()));
        self.input_box.set_visible(true);
        self.input_box.set_focus();
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
            let hwnd = self.popup_window.handle.hwnd().unwrap();
            let theme : Vec<u16> = "Explorer".as_bytes().iter().map(|x| {*x as u16}).collect();
            SetWindowTheme(hwnd, theme.as_ptr(), 0 as LPCWSTR);
            let mut style = GetWindowLongA(hwnd, GWL_STYLE) as u32;
            style |= WS_EX_TOOLWINDOW;   // flags don't work - windows remains in taskbar
            style &= !(WS_EX_APPWINDOW);
            ShowWindow(hwnd, SW_HIDE); // hide the window
            SetWindowLongPtrA(hwnd, GWL_STYLE, style as isize); // set the style
            
        }
        self.tray_notification();
        self.expect_shortcut();
        
        let dv = &self.pass_view;

        dv.insert_column("Name");
        dv.insert_column("Description");
        dv.insert_column("Affiliated apps");
        dv.set_headers_enabled(true);
        self.update_list();
    }

    fn update_list(&self) {
        let pt = self.passtable.borrow();
        let mut names: Vec<&String> = pt.get_names().collect();
        names.sort();
        self.update_list_with_names(&names);
    }

    fn update_list_with_names(&self, names: &[&String]) {
        *self.pass_names.borrow_mut() = names.iter().map(|x| {(*x).clone()}).collect(); 

        let dv = &self.pass_view;
        dv.clear();
        let pt = self.passtable.borrow();
        for name in &(*self.pass_names.borrow()) {
            let meta = pt.get_metadata(name).unwrap();
            let ind: i32 = dv.len() as i32;
            dv.insert_item(nwg::InsertListViewItem {
                index: Some(ind),
                column_index: 0,
                text: Some(name.clone()),
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
        self.passtable.borrow().to_file(&path)
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
    let _app = PassToolApp::build_ui(PassToolApp{passtable: RefCell::new(pt), ..Default::default()}).expect("Failed to build UI");

    nwg::dispatch_thread_events();
}