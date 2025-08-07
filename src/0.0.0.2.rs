use gio::ffi::G_FILE_MONITOR_EVENT_CREATED;
use glib::ControlFlow;
use gtk4::gdk::Display;
use gtk4::gdk_pixbuf::PixbufLoader;
use gtk4::prelude::WidgetExtManual;
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, DrawingArea, EventControllerMotion, GestureClick, GestureDrag, Image,
    Label, Orientation,
};
use gtk4::{CssProvider, StyleContext};
use libadwaita::prelude::*;
use libadwaita::{Application as AdwApplication, ApplicationWindow, HeaderBar};
use std::cell::RefCell;
use std::rc::Rc; // これも必要

const ICON_DATA: &[u8] = include_bytes!("../icon.svg");

// アイコンをバイナリに含める関数
fn load_embedded_logo() -> Image {
    let loader = PixbufLoader::new();
    loader.write(ICON_DATA).unwrap();
    loader.close().unwrap();

    let pixbuf = loader.pixbuf().unwrap();
    let image = Image::from_pixbuf(Some(&pixbuf));
    image.set_pixel_size(24);
    image
}

fn apply_hover_effects(label: &Label, is_clicked: Rc<RefCell<bool>>) {
    let label_hover = label.clone();
    let is_clicked_hover = is_clicked.clone();

    let motion_controller = EventControllerMotion::new();
    motion_controller.connect_enter(move |_, _, _| {
        if !*is_clicked_hover.borrow() {
            label_hover.add_css_class("file-label-hover");
        }
    });

    let label_leave = label.clone();
    let is_clicked_leave = is_clicked.clone();
    motion_controller.connect_leave(move |_| {
        if !*is_clicked_leave.borrow() {
            label_leave.remove_css_class("file-label-hover");
        }
    });

    label.add_controller(motion_controller);
}

fn main() {
    // ==== UIの情報
    let preview_height = 500.0;
    let preview_height_int = preview_height as i32;

    // ==== Adwaitaアプリケーションを作成
    let app = AdwApplication::builder()
        .application_id("com.Luvita.app")
        .build();

    app.connect_activate(move |app| {
        // CSSでヘッダーバーの高さを細くする
        let css = "
            headerbar, .titlebar {
                min-height: 24px;
                /* padding-top: 0;
                padding-bottom: 0;
                 background-color:rgb(0, 110, 255);  明るめの青 */
            }

            headerbar * {
                margin-top: 0;
                margin-bottom: 0;
                /* color: white;  ボタンや文字が見えるように */
            }

            .file-label {
                background-color: transparent;
                margin: 3px -3px 2px -3px;
                padding: 0 10px 0 10px;
            }

            .file-label-hover {
                background-color: rgba(151, 151, 151, 0.24); /* 半透明な青 */
                border-radius: 8px
            }

            .file-label-clicked {
                background-color: rgba(0, 122, 204, 1.0); /* 濃い青 */
                color: white;
            }
        ";
        let provider = CssProvider::new();
        provider.load_from_data(css);

        StyleContext::add_provider_for_display(
            &Display::default().expect("Display not found"),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // Adwaitaのウィンドウを作成
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Luvita")
            .default_width(1305)
            .default_height(800)
            .build();

        // ==== ウィンドウサイズを取得
        let last_size = Rc::new(RefCell::new((0, 0)));
        let last_size_clone = last_size.clone();
        window.add_tick_callback(move |win, _| {
            let alloc = win.allocation();
            let (w, h) = (alloc.width(), alloc.height());
            let mut last = last_size_clone.borrow_mut();
            if w != last.0 || h != last.1 {
                println!("Window resized: width = {}, height = {}", w, h);
                *last = (w, h);
            }
            ControlFlow::Continue
        });

        // playheadの位置
        let mut playhead_position = Rc::new(RefCell::new((200.0, 150.0)));

        // ==== プレビューエリア
        let preview_area = DrawingArea::builder()
            .content_width(800)
            .content_height(preview_height_int)
            .build();

        // タイムラインエリア
        let timeline_area = DrawingArea::builder()
            .content_width(800)
            .content_height(800)
            .build();

        {
            let playhead = playhead_position.clone();
            timeline_area.set_draw_func(move |drawing_area, cr, width, height| {
                let top = 400.0;

                println!(
                    "Window height: {}, drawing rectangle from y={} to bottom",
                    height, top
                );

                // 背景をダークグレーで塗りつぶし
                cr.set_source_rgb(0.1, 0.1, 0.1); // 0.0~1.0の範囲でRGB指定、暗めのグレー
                cr.paint().unwrap();

                // 横線とレイヤーラベルを30px間隔で描画
                let layer_height = 30.0;
                let label_area_width = 40.0; // 左のラベル描画幅
                cr.set_source_rgb(0.3, 0.3, 0.3);
                cr.set_line_width(1.0);

                let num_layers = (height as f64 / layer_height).ceil() as i32;
                let top_offset = &preview_height;
                for i in 0..num_layers {
                    let y = i as f64 * layer_height + top_offset;

                    // 横線
                    cr.move_to(0.0, y);
                    cr.line_to(width as f64, y);
                    cr.stroke().unwrap();

                    // レイヤー番号テキストを左に描画
                    let label = format!(" {}", i + 1);
                    cr.set_font_size(14.0);
                    cr.move_to(5.0, y + 14.0); // 5px右、14px下に調整（フォントサイズ考慮）
                    cr.set_source_rgb(0.8, 0.8, 0.8);
                    // cr.set_source_rgb(1.0, 1.0, 1.0);
                    cr.show_text(&label).unwrap();

                    // ラベルの右に縦線（縦棒）
                    cr.set_source_rgb(0.3, 0.3, 0.3);
                    cr.move_to(label_area_width, y);
                    cr.line_to(label_area_width, y + layer_height);
                    cr.stroke().unwrap();
                }

                // マウスの座標を取得
                let (x, y) = *playhead.borrow();

                // Playheadを描画
                cr.set_source_rgb(0.0, 0.8, 1.0); // 水色
                cr.set_line_width(1.0); // 線の太さ
                cr.move_to(x, preview_height); // 縦線の開始位置
                cr.line_to(x, preview_height + 300.0); // 縦線の終了位置
                let _ = cr.stroke(); // 描画実行

                // マウスの動きに反応
                let motion = EventControllerMotion::new();
                let playhead_position_for_motion = playhead.clone();
                //let timeline_area_for_motion = drawing_area.clone();

                motion.connect_motion(move |_, x, y| {
                    *playhead_position_for_motion.borrow_mut() = (x, y); // マウスが動いているときの座標
                    //timeline_area_for_motion.queue_draw();
                });

                drawing_area.add_controller(motion);
                println!("{}", playhead.borrow().0);
            });
        }

        // playheadを右クリック中は追従させる
        let drag = GestureDrag::new();
        let drag_offset = Rc::new(RefCell::new((0.0, 0.0)));

        let playhead_position_for_begin = playhead_position.clone();
        let playhead_position_for_end = playhead_position.clone();
        let drag_offset_for_begin = drag_offset.clone();
        let drag_offset_for_end = drag_offset.clone();

        drag.connect_drag_begin(move |_, start_x, start_y| {
            let (cx, cy) = *playhead_position_for_begin.borrow();
            drag_offset_for_begin.borrow_mut().0 = start_x - cx;
            drag_offset_for_begin.borrow_mut().1 = start_y - cy;
        });

        drag.connect_drag_end(move |_, end_x, end_y| {
            let (cx, cy) = *playhead_position_for_end.borrow();
            drag_offset_for_end.borrow_mut().0 = end_x - cx;
            drag_offset_for_end.borrow_mut().1 = end_y - cy;
            println!("endpoint: {}", end_x) //end_xは移動距離だった
        });

        let playhead_position_for_update = playhead_position.clone();
        let drag_offset_for_update = drag_offset.clone();
        let drawing_area_for_update = timeline_area.clone();

        drag.connect_drag_update(move |_, offset_x, offset_y| {
            let dx = offset_x - drag_offset_for_update.borrow().0;
            let dy = offset_y - drag_offset_for_update.borrow().1;
            *playhead_position_for_update.borrow_mut() = (dx, dy);
            drawing_area_for_update.queue_draw();
            // println!(
            //     "playhead_position: {}",
            //     playhead_position_for_update.borrow().0
            // ) //end_xは移動距離だった
        });
        timeline_area.add_controller(drag);

        // 右クリックの挙動を検知
        let click = GestureClick::builder().button(3).build(); //左1 中央2 右3
        click.connect_pressed(move |_, n_press, x, y| {
            println!("右クリック検出: ({}, {}) クリック回数: {}", x, y, n_press);
        });
        click.connect_released(move |_, n_press, x, y| {
            println!(
                "右クリックが離された！ 座標: ({}, {}), クリック回数: {}",
                x, y, n_press
            );
        });
        timeline_area.add_controller(click);

        // ==== ヘッダーバー
        let header_box = GtkBox::new(Orientation::Horizontal, 6);
        header_box.set_halign(Align::Start);

        // ← 追加ここから
        //let file_label = Label::new(Some("File"));
        //file_label.set_margin_start(12); // 少し余白（任意）
        let file_label = Label::new(Some("ファイル"));
        //file_label.set_margin_start(12); //最初にスペース
        file_label.set_focusable(true);
        file_label.add_css_class("file-label");

        let filter_label = Label::new(Some("フィルタ"));
        filter_label.set_focusable(true);
        filter_label.add_css_class("file-label");

        let setting_label = Label::new(Some("設定"));
        setting_label.set_focusable(true);
        setting_label.add_css_class("file-label");

        let is_clicked = Rc::new(RefCell::new(false));
        let file_label_clone = file_label.clone();
        let is_clicked_clone = is_clicked.clone();

        // --- マウスオーバー検出 ---
        let motion_controller = EventControllerMotion::new();
        let file_label_for_hover = file_label.clone();

        let is_clicked_clone = is_clicked.clone();
        motion_controller.connect_enter(move |_, x, y| {
            if !*is_clicked_clone.borrow() {
                file_label_for_hover.add_css_class("file-label-hover");
            }
        });
        let file_label_for_leave = file_label.clone();
        let is_clicked_for_leave = is_clicked.clone();

        motion_controller.connect_leave(move |_| {
            if !*is_clicked_for_leave.borrow() {
                file_label_for_leave.remove_css_class("file-label-hover");
            }
        });

        file_label.add_controller(motion_controller);

        // --- クリック処理 ---
        let gesture = GestureClick::builder().button(1).build();
        let file_label_for_click = file_label.clone();
        let is_clicked_for_click = is_clicked.clone();

        gesture.connect_pressed(move |_, _, _, _| {
            let mut clicked = is_clicked_for_click.borrow_mut();
            *clicked = !*clicked;

            if *clicked {
                file_label_for_click.remove_css_class("file-label-hover");
                file_label_for_click.add_css_class("file-label-clicked");
            } else {
                file_label_for_click.remove_css_class("file-label-clicked");
            }
        });

        file_label.add_controller(gesture);
        //header_box.append(&file_label);

        // ====== ここで他の場所のクリックを検出 ======
        let file_label_for_outside = file_label.clone();
        let is_clicked_for_outside = is_clicked.clone();

        let global_click = GestureClick::builder().button(1).build();
        global_click.connect_pressed(move |_, _, _, _| {
            if *is_clicked_for_outside.borrow() {
                file_label_for_outside.remove_css_class("file-label-clicked");
                *is_clicked_for_outside.borrow_mut() = false;
            }
        });
        window.add_controller(global_click);

        apply_hover_effects(&file_label, is_clicked.clone());
        apply_hover_effects(&filter_label, Rc::new(RefCell::new(false)));
        apply_hover_effects(&setting_label, Rc::new(RefCell::new(false)));
        // → 追加ここまで

        // Set Title "Luvita"
        // let title_label = Label::new(Some("Luvita"));
        // title_label.set_markup("<b>Luvita</b>");
        // header_box.append(&title_label);

        // Icon
        let icon = Image::from_file("icon.svg");
        icon.set_pixel_size(20); // アイコンサイズ調整
        icon.set_margin_start(5); // マージンを追加してロゴを少し右に移動
        icon.set_margin_end(5);

        let header = HeaderBar::builder()
            .title_widget(&header_box)
            .show_end_title_buttons(true)
            .build();

        // アイコンをバイナリに含める
        let loader = PixbufLoader::new();
        loader.write(ICON_DATA).unwrap();
        loader.close().unwrap();

        let pixbuf = loader.pixbuf().unwrap();
        let logo = Image::from_pixbuf(Some(&pixbuf));
        logo.set_pixel_size(24); // 必要ならサイズ調整

        // 左端に保存ボタンを配置
        header.pack_start(&icon);
        header.pack_start(&file_label);
        header.pack_start(&filter_label);
        header.pack_start(&setting_label);

        // ==== UI組み立て ====
        let vbox = GtkBox::new(Orientation::Vertical, 0);
        vbox.append(&header);
        vbox.append(&timeline_area);
        window.set_content(Some(&vbox));
        window.present();
    });

    app.run();
}

// playheadの座標を保持し、ドラッグできるようにする。
