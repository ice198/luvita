use gtk4::{Align, Box as GtkBox, Button, Label, Orientation, Switch};
use libadwaita::prelude::*;
use libadwaita::{Application, ApplicationWindow, HeaderBar};

fn main() {
    let app = Application::builder()
        .application_id("com.example.myapp")
        .build();

    app.connect_activate(|app| {
        let win = ApplicationWindow::builder()
            .application(app)
            .title("Luvita")
            .default_width(800)
            .default_height(600)
            .build();

        // ラベル + スイッチ を横並びにするBox（HeaderBar用）
        let header_box = GtkBox::new(Orientation::Horizontal, 6);
        header_box.set_halign(Align::Start); // 左寄せ

        let title_label = Label::new(Some("Luvita"));
        title_label.set_markup("<b>Luvita</b>");

        let switch = build_switch();

        header_box.append(&title_label);
        header_box.append(&switch);

        let header = HeaderBar::builder()
            .title_widget(&header_box)
            .show_end_title_buttons(true)
            .build();

        let vbox = GtkBox::new(Orientation::Vertical, 0);
        vbox.append(&header);

        // メインの表示内容
        let content = Label::new(Some("こんにちは"));

        // 表示切り替え用スイッチ
        let content_switch = Switch::builder().halign(Align::Start).build();
        content_switch.set_active(true);

        let content_clone = content.clone();
        content_switch.connect_active_notify(move |s| {
            let visible = s.is_active();
            content_clone.set_visible(visible);
        });

        let hbox = GtkBox::new(Orientation::Horizontal, 5);
        hbox.append(&Label::new(Some("表示:")));
        hbox.append(&content_switch);

        vbox.append(&hbox);
        vbox.append(&content);

        win.set_content(Some(&vbox));
        win.present();
    });

    app.run();
}

// デバッグ用のスイッチ（タイトルバー内）
fn build_switch() -> Switch {
    let switch = Switch::builder().build();
    switch.connect_active_notify(|s| {
        println!("state changed: {:?}", s.is_active());
    });
    switch
}

-------------------------------test2

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Button, HeaderBar};
use libadwaita::prelude::*;
use libadwaita::{Application, ApplicationWindow, HeaderBar};

fn main() {
    // GTKアプリケーションを作成
    let app = Application::new(
        Some("com.example.headerbar_save_button"),
        Default::default(),
    );

    app.connect_activate(|app| {
        // メインウィンドウ作成
        let window = ApplicationWindow::new(app);
        window.set_title(Some("HeaderBar Save Button Example"));
        window.set_default_size(400, 200);

        // ヘッダーバー作成
        let header = HeaderBar::new();
        header.set_show_title_buttons(true);

        // 保存ボタンを作成
        let save_button = Button::with_label("保存");
        // 左端にボタンを配置
        header.pack_start(&save_button);

        // ウィンドウにヘッダーバーをセット
        window.set_titlebar(Some(&header));

        // 保存ボタンがクリックされた時の動作
        save_button.connect_clicked(|_| {
            println!("保存ボタンがクリックされました！");
        });

        window.show();
    });

    app.run();
}

---------------------------------------------- マウスで戦を書く

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, DrawingArea, GestureDrag};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let app = Application::builder()
        .application_id("com.example.drawline")
        .build();

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("マウスで線を描く")
            .default_width(800)
            .default_height(600)
            .build();

        // 座標を記録する可変なベクタ（Vec<(x, y)>）
        let points: Rc<RefCell<Vec<(f64, f64)>>> = Rc::new(RefCell::new(Vec::new()));

        let drawing_area = DrawingArea::builder()
            .content_width(800)
            .content_height(600)
            .build();

        // 描画関数：記録された座標に従って線を描く
        {
            let points = points.clone();
            drawing_area.set_draw_func(move |_, cr, _, _| {
                let points = points.borrow();
                if points.len() < 2 {
                    return;
                }

                cr.set_source_rgb(0.0, 0.0, 0.0); // 黒色
                cr.set_line_width(2.0);

                // 最初の点からスタート
                let mut iter = points.iter();
                if let Some((x, y)) = iter.next() {
                    cr.move_to(*x, *y);
                    for (x, y) in iter {
                        cr.line_to(*x, *y);
                    }
                    cr.stroke().expect("stroke failed");
                }
            });
        }

        // マウスドラッグのジェスチャ
        let drag = GestureDrag::new();
        {
            let points = points.clone();
            let da = drawing_area.clone();
            drag.connect_drag_update(move |_gesture, offset_x, offset_y| {
                points.borrow_mut().push((offset_x, offset_y));
                da.queue_draw(); // 再描画を要求
            });
        }

        // ドラッグ開始時に座標を初期化
        {
            let points = points.clone();
            drag.connect_drag_begin(move |_gesture, start_x, start_y| {
                points.borrow_mut().clear();
                points.borrow_mut().push((start_x, start_y));
            });
        }

        drawing_area.add_controller(drag);
        window.set_child(Some(&drawing_area));
        window.present();
    });

    app.run();
}

// drawareaのうえにボタンを設置
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Button, DrawingArea, Overlay};
use libadwaita::{Application as AdwApplication, ApplicationWindow as AdwWindow};

fn main() {
    let app = AdwApplication::builder()
        .application_id("com.Luvita.app")
        .build();

    app.connect_activate(|app| {
        // AdwWindowを作成
        let window = ApplicationWindow::new(app); // libadwaitaのAdwWindowを使用
        window.set_default_size(800, 600);

        // Overlayウィジェットを作成
        let overlay = Overlay::new();

        // DrawingAreaを作成
        let drawing_area = DrawingArea::new();
        drawing_area.set_size_request(800, 600);

        // DrawingAreaの描画設定
        drawing_area.set_draw_func(|_, cr, width, height| {
            // カスタム描画を行う
            cr.set_source_rgb(0.0, 0.0, 1.0); // 青色
            cr.paint().expect("Unable to paint");
        });

        // ボタンを作成
        let button = Button::with_label("Click me");
        button.set_size_request(200, 100);

        // DrawingAreaをOverlayに追加
        overlay.add_overlay(&drawing_area);

        // ボタンをOverlayに追加
        overlay.add_overlay(&button);

        // AdwWindowにOverlayをセット
        window.set_child(Some(&overlay));

        // ウィンドウを表示
        window.show();
    });

    app.run();
}
