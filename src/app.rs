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
    #[nwg_events(OnKeyEnter: [PassToolApp::get_password], OnKeyEsc: [PassToolApp::toggle_overlay])]
    popup_window: nwg::Window, // main popup window

    #[nwg_control(spacing: 15)]
    #[nwg_layout(parent: popup_window)]
    layout: nwg::GridLayout, // layout

    #[nwg_control(text: "App-specific passwords:", flags: "VISIBLE")]
    #[nwg_layout_item(layout: layout, row: 0, col_span: 4)]
    label1: nwg::Label,
    
    #[nwg_control(item_count: 10, list_style: nwg::ListViewStyle::Detailed, focus: false,
        ex_flags: nwg::ListViewExFlags::GRID | nwg::ListViewExFlags::FULL_ROW_SELECT, 
        background_color: [100,100,100]
    )]
    #[nwg_layout_item(layout: layout, col_span: 4, row: 1, row_span: 3)]
    #[nwg_events(OnListViewItemActivated: [PassToolApp::enable_input(SELF, CTRL)], OnListViewItemChanged: [PassToolApp::disable_input], OnListViewRightClick: [PassToolApp::update_selected_password(SELF,CTRL), PassToolApp::show_edit_menu])]
    rec_pass_view: nwg::ListView, // list of app-specific passwords
    rec_pass_names: RefCell<Vec<String>>,

    active_process: RefCell<String>, // process name below the popup window
    
    #[nwg_control(text: "All passwords:", flags: "VISIBLE")]
    #[nwg_layout_item(layout: layout, row: 4, col_span: 3)]
    label2: nwg::Label,
    #[nwg_control(text: "Password's apps:", flags: "VISIBLE")]
    #[nwg_layout_item(layout: layout, row: 4, col: 3, col_span: 2)]
    label3: nwg::Label,
    #[nwg_control(item_count: 10, list_style: nwg::ListViewStyle::Detailed, focus: true,
        ex_flags: nwg::ListViewExFlags::GRID | nwg::ListViewExFlags::FULL_ROW_SELECT, 
        background_color: [100,100,100]
    )]
    #[nwg_layout_item(layout: layout, col_span: 3, row: 5, row_span: 7)]
    #[nwg_events(OnListViewItemActivated: [PassToolApp::enable_input(SELF, CTRL)], OnListViewItemChanged: [PassToolApp::disable_input], OnListViewRightClick: [PassToolApp::update_selected_password(SELF,CTRL), PassToolApp::show_edit_menu])]
    pass_view: nwg::ListView, // list of passwords
    pass_names: RefCell<Vec<String>>,
        
    #[nwg_control(parent: popup_window, text: "Add app")]
    #[nwg_layout_item(layout: layout, row: 5, col: 3, col_span: 2, row_span: 2)]
    #[nwg_events(OnButtonClick: [])]
    add_app_button: nwg::Button,
    #[nwg_control(item_count: 10, list_style: nwg::ListViewStyle::Simple, focus: false,
        ex_flags: nwg::ListViewExFlags::GRID | nwg::ListViewExFlags::FULL_ROW_SELECT, 
        background_color: [100,100,100]
    )]
    #[nwg_layout_item(layout: layout, col: 3, col_span: 2, row: 7, row_span: 6)]
    #[nwg_events()]
    app_view: nwg::ListView,
    app_names: RefCell<Vec<String>>,

    #[nwg_control(parent: popup_window, text: "", placeholder_text: Some("Choose password"), password: Some('*'), flags: "VISIBLE")]
    #[nwg_layout_item(layout: layout, col_span: 3, row: 12, row_span: 1)]
    key_input: nwg::TextInput, // input 
    key_label: RefCell<String>, // shared string

    selected_password: RefCell<Option<String>>,

    #[nwg_control(parent: popup_window, text: "New")]
    #[nwg_layout_item(layout: layout, row: 1, col: 4, col_span: 1, row_span: 3)]
    #[nwg_events(OnButtonClick: [PassToolApp::show_add_password])]
    new_button: nwg::Button,

    #[nwg_control(parent: Some(&data.popup_window), title: "Add/Edit password", size: (300, 250), flags: "WINDOW|DISABLED")]
    #[nwg_events(OnWindowClose: [PassToolApp::clear_add_password])]
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

    #[nwg_control(parent: popup_window, popup: true)]
    edit_menu: nwg::Menu, // tray menu
    
    #[nwg_control(parent: edit_menu, text: "Edit")]
    #[nwg_events(OnMenuItemSelected: [PassToolApp::edit_password])]
    edit_item1: nwg::MenuItem, // tray menu option

    #[nwg_control(parent: edit_menu, text: "Delete")]
    #[nwg_events(OnMenuItemSelected: [PassToolApp::remove_password])]
    edit_item2: nwg::MenuItem, // tray menu option

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
    fn remove_password(&self) {
        let name = self.selected_password.borrow();
        if name.is_none() {return}

        let name = name.as_ref().unwrap();
        let confirm_password_delete = nwg::MessageParams {
            title: "Delete password",
            content: &format!("Are you sure you want to delete the password \'{name}\'?"),
            buttons: nwg::MessageButtons::YesNo,
            icons: nwg::MessageIcons::Warning
        };
        if let nwg::MessageChoice::Yes = nwg::modal_message(self.popup_window.handle, &confirm_password_delete)
        {
            let _ = self.passtable.borrow_mut().remove_password(name);
        }

        let _ = self.save();
        self.update_lists();
    }

    fn get_password(&self) {
        let name = self.selected_password.borrow();
        if name.is_none() {return}

        let name = name.as_ref().unwrap();
        let key = self.key_input.text();
        if key.len() == 0{
            self.key_input.set_enabled(true);
            self.key_input.set_focus();
            return;
        }
        self.key_input.set_text("");
        match self.passtable.borrow().get_password(name, &key) {
            Ok(password) => {
                nwg::Clipboard::set_data_text(self.popup_window.handle, &password);
                nwg::modal_info_message(self.popup_window.handle, "Success!","Password saved into clipboard!");
                self.disable_input();
            }
            Err(passtool::IncorrectPass) => {
                nwg::modal_error_message(self.popup_window.handle, "Warning!", "Incorrect password!");
                self.key_input.set_focus();
            },
            Err(e) => {nwg::modal_error_message(self.popup_window.handle, "Unknown error!", &format!("{e}"));}
        }
    }
    
    fn show_edit_menu(&self) {
        let name = self.selected_password.borrow();
        if name.is_none() {return}
        
        let (x, y) = nwg::GlobalCursor::position();
        self.edit_menu.popup(x, y);
    }

    fn edit_password(&self) {
        let name = self.selected_password.borrow();
        if name.is_none() {return}
        
        {
            let pt = self.passtable.borrow();
            let name = name.as_ref().unwrap();
            let meta = pt.get_metadata(name).unwrap();
            
            self.password_name_input.set_text(name);
            self.password_description_input.set_text(&meta.description);
        }
    
        self.show_add_password();
    }

    fn add_password(&self) {
        let name = self.password_name_input.text();
        let password = self.password_input.text();
        let key = self.password_key_input.text();
        let rep_key = self.password_repeat_key_input.text();
        let description = self.password_description_input.text();

        let confirm_password_edit = nwg::MessageParams {
            title: "Edit password",
            content: &format!("Password with this name already exists. Are you sure you want to edit the password \'{name}\'?"),
            buttons: nwg::MessageButtons::YesNo,
            icons: nwg::MessageIcons::Warning
        };
        
        { // additional scope for pt
            let mut pt = self.passtable.borrow_mut();
            if name.len() == 0 {
                nwg::modal_error_message(self.add_password_window.handle, "Warning!", "Empty name is not allowed!");
                return;
            }
            if password.len() == 0 {
                if !pt.contains(&name) {
                    nwg::modal_error_message(self.add_password_window.handle, "Warning!", "Empty password is not allowed!");
                    return;
                }
                else{
                    if key.len() != 0 || rep_key.len() != 0 {
                        nwg::modal_error_message(self.add_password_window.handle, "Warning!", "It's impossible to update password without specifying the key and vice versa!");
                        return;
                    }
    
                    if let nwg::MessageChoice::Yes = nwg::modal_message(self.popup_window.handle, &confirm_password_edit)
                    {
                        let old_apps = pt.get_metadata(&name).unwrap().apps.clone();
                        let _ = pt.update_metadata(&name, PasswordMeta::new(description, old_apps));
                    }
                    else {return}
                }
            }
            else{
                if key.len() == 0 {
                    nwg::modal_error_message(self.add_password_window.handle, "Warning!", "Empty key is not allowed!");
                    return;
                }
                if key != rep_key{
                    nwg::modal_error_message(self.add_password_window.handle, "Warning!", "Keys do not match!", );
                    return;
                }
                if pt.contains(&name){
                    if let nwg::MessageChoice::Yes = nwg::modal_message(self.popup_window.handle, &confirm_password_edit) {
                        let _ = pt.remove_password(&name);
                    }
                    else {return}
                }
                let _ = pt.add_password(&name, &password, PasswordMeta::new(description, Default::default()), &key);
            }
        }

        let _ = self.save();
        self.clear_add_password();
        self.add_password_window.set_visible(false);
        self.update_lists();
    }

    fn clear_add_password(&self) {
        self.password_name_input.set_text("");
        self.password_input.set_text("");
        self.password_key_input.set_text("");
        self.password_repeat_key_input.set_text("");
        self.password_description_input.set_text("");
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
                *self.active_process.borrow_mut() = process_name.iter().filter_map(|x| if *x != 0 { Some(char::from_u32(*x as u32).unwrap()) } else { None }).collect(); //setting active_process to the name of the underlying window process
                
                //dbg!(self.active_process.borrow());

                let (w, h) = self.popup_window.size();
                let (w, h) = (w as i32, h as i32);
                let [total_width, total_height] = [nwg::Monitor::width(), nwg::Monitor::height()];
                
                x = std::cmp::min(total_width-w, x-w/2);
                y = std::cmp::min(total_height-h-50, y-h/2);

                self.popup_window.set_position(x, y);
                self.update_lists();
                self.popup_window.set_enabled(true);
                self.popup_window.set_visible(true);

                if self.rec_pass_names.borrow().len() != 0 { self.rec_pass_view.set_focus() } 
                else { self.pass_view.set_focus(); }
            }
            else { 
                self.add_password_window.set_visible(false);
                self.popup_window.set_visible(false); 
            }
        }
    }

    fn update_selected_password(&self, view: &nwg::ListView) {
        let ind = view.selected_item();
        if ind.is_none() {
            *self.selected_password.borrow_mut() = None;
            return
        }
        let ind = ind.unwrap();
        let pass_names = if view.handle == self.pass_view.handle {self.pass_names.borrow()} else {self.rec_pass_names.borrow()};
        let pass_name = pass_names[ind].clone();
        *self.selected_password.borrow_mut() = Some(pass_name);
    }

    fn disable_input(&self) {
        self.key_input.set_text("");
        self.key_input.set_placeholder_text(Some("Choose password"));
        self.key_input.set_enabled(false);
        self.key_input.set_visible(true);
    }

    fn enable_input(&self, view: &nwg::ListView) {
        self.update_selected_password(view);
        let selected_password = self.selected_password.borrow();
        let pass_name = (*selected_password).as_ref().unwrap();
        *self.key_label.borrow_mut() = format!("Input key for the password \"{pass_name}\":");

        self.key_input.set_enabled(true);
        self.key_input.set_placeholder_text(Some(self.key_label.borrow_mut().as_str()));
        self.key_input.set_visible(true);
        self.key_input.set_focus();
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
        self.key_input.set_visible(true);
        self.key_input.set_enabled(false);
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
        
        let pv = &self.pass_view;
        pv.insert_column("Name");
        pv.insert_column("Description");
        pv.set_headers_enabled(true);

        let rpv = &self.rec_pass_view;
        rpv.insert_column("Name");
        rpv.insert_column("Description");
        rpv.set_headers_enabled(true);

        let av = &self.app_view;
        av.insert_column("Path");
        self.update_lists();
    }

    fn update_lists(&self) {
        self.update_rec_passwords();
        self.update_all_passwords();
    }

    fn update_rec_passwords(&self) {
        let app = &*self.active_process.borrow();
        let pt = self.passtable.borrow();
        let mut names: Vec<&String> = pt.get_names().collect();
        names.sort();
        *self.rec_pass_names.borrow_mut() = names.iter()
                                                .map(|x| {(*x).clone()})
                                                .filter(|x| {
                                                    pt.get_metadata(x).unwrap().apps.contains(app)
                                                }).collect();
        let rv = &self.rec_pass_view;
        rv.clear();
        for name in &(*self.rec_pass_names.borrow()) {
            let meta = pt.get_metadata(name).unwrap();
            let ind: i32 = rv.len() as i32;
            rv.insert_item(nwg::InsertListViewItem {
                index: Some(ind),
                column_index: 0,
                text: Some(name.clone()),
                image: None,
            });
            rv.insert_item(nwg::InsertListViewItem {
                index: Some(ind),
                column_index: 1,
                text: Some(meta.description.clone()),
                image: None,
            }); 
        }
    }

    fn update_all_passwords(&self) {
        let pt = self.passtable.borrow();
        let mut names: Vec<&String> = pt.get_names().collect();
        names.sort();
        *self.pass_names.borrow_mut() = names.iter().map(|x| {(*x).clone()}).collect(); 

        let dv = &self.pass_view;
        dv.clear();
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