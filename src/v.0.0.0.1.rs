use glib::ControlFlow;
use gtk4::gdk_pixbuf::PixbufLoader;
use gtk4::gdk::Display;
use gtk4::prelude::WidgetExtManual;
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, DrawingArea, GestureClick, GestureDrag, Image, Label,
    Orientation, Switch, CssProvider, EventControllerMotion,
};
use libadwaita::prelude::*;
use libadwaita::{Application as AdwApplication, ApplicationWindow, HeaderBar};
use std::cell::RefCell;
use std::rc::Rc;

const ICON_DATA: &[u8] = include_bytes!(".././save_icon.svg");

fn load_embedded_logo() -> Image {
    let loader = PixbufLoader::new();
    loader.write(ICON_DATA).unwrap();
    loader.close().unwrap();

    let pixbuf = loader.pixbuf().unwrap();
    let image = Image::from_pixbuf(Some(&pixbuf));
    image.set_pixel_size(24);
    image
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

        //apply_custom_css();  // CSSを読み込んでいたが廃止

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

        // ==== 円の位置 ====
        let circle_pos = Rc::new(RefCell::new((200.0, 150.0)));
        // Playheadの位置を保持する変数
        let playhead_position = circle_pos.clone();

        // ==== 円の色（RGBA）====
        // 通常色
        let normal_color = (0.2235, 1.0, 0.0784);
        // 青色
        let active_color = (0.0, 0.0, 1.0);
        // 現在の色を保持
        let circle_color = Rc::new(RefCell::new(normal_color));

        // ==== プレビューエリア
        let preview_area = DrawingArea::builder()
            .content_width(800)
            .content_height(preview_height_int)
            .build();

        let drawing_area = DrawingArea::builder()
            .content_width(800)
            .content_height(800)
            .build();

        {
            let circle_pos = circle_pos.clone();
            let circle_color = circle_color.clone();
            drawing_area.set_draw_func(move |drawing_area, cr, width, height| {
                let top = 400.0;
                let rect_height = (height as f64 - top).max(0.0); // 負にならないように

                println!(
                    "Window height: {}, drawing rectangle from y={} to bottom",
                    height, top
                );

                cr.set_source_rgb(0.2, 0.6, 0.8); // 青系
                cr.rectangle(0.0, top, width as f64, rect_height);
                cr.fill().unwrap();
                // 背景：チェッカーボード
                // let tile_size = 10.0;
                // for i in 0..((width as f64 / tile_size).ceil() as i32) {
                //     for j in 0..((height as f64 / tile_size).ceil() as i32) {
                //         let is_light = (i + j) % 2 == 0;
                //         let gray = if is_light { 0.85 } else { 0.65 };
                //         cr.set_source_rgb(gray, gray, gray);
                //         cr.rectangle(
                //             i as f64 * tile_size,
                //             j as f64 * tile_size,
                //             tile_size,
                //             tile_size,
                //         );
                //         cr.fill().unwrap();
                //     }
                // }

                // プレビューを描画
                cr.set_source_rgb(1.0, 0.1, 0.1); // 0.0~1.0の範囲でRGB指定、暗めのグレー
                cr.paint().unwrap();

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

                // 円の描画
                let (x, y) = *circle_pos.borrow();
                // let (r, g, b) = *circle_color.borrow();
                // let radius = 50.0;

                // cr.set_source_rgb(r, g, b);
                // cr.arc(
                //     x + radius,
                //     y + radius,
                //     radius,
                //     0.0,
                //     std::f64::consts::PI * 2.0,
                // );
                // cr.fill().unwrap();

                // Playheadを描画
                cr.set_source_rgb(0.0, 0.8, 1.0); // 赤色
                cr.set_line_width(1.0); // 線の太さ
                cr.move_to(x, preview_height); // 縦線の開始位置
                cr.line_to(x, preview_height + 300.0); // 縦線の終了位置
                let _ = cr.stroke(); // 描画実行

                let (x, y) = *playhead_position.borrow();
                // cr.set_source_rgb(0.2, 0.8, 1.0); // 円の色
                // cr.arc(x, y, 20.0, 0.0, std::f64::consts::PI * 2.0);
                // cr.fill().unwrap();
                // マウスの動きに反応
                let motion = EventControllerMotion::new();
                let circle_pos_for_motion = circle_pos.clone();
                let drawing_area_for_motion = drawing_area.clone();

                motion.connect_motion(move |_, x, y| {
                    *circle_pos_for_motion.borrow_mut() = (x, y);
                    drawing_area_for_motion.queue_draw();
                });

                drawing_area.add_controller(motion);

            });
        }

        // ==== ドラッグ操作 ====
        let drag = GestureDrag::new();
        let drag_offset = Rc::new(RefCell::new((0.0, 0.0)));

        let circle_pos_for_begin = circle_pos.clone();
        let drag_offset_for_begin = drag_offset.clone();

        drag.connect_drag_begin(move |_, start_x, start_y| {
            let (cx, cy) = *circle_pos_for_begin.borrow();
            drag_offset_for_begin.borrow_mut().0 = start_x - cx;
            drag_offset_for_begin.borrow_mut().1 = start_y - cy;
        });

        let circle_pos_for_update = circle_pos.clone();
        let drag_offset_for_update = drag_offset.clone();
        let drawing_area_for_update = drawing_area.clone();

        drag.connect_drag_update(move |_, offset_x, offset_y| {
            let dx = offset_x - drag_offset_for_update.borrow().0;
            let dy = offset_y - drag_offset_for_update.borrow().1;
            *circle_pos_for_update.borrow_mut() = (dx, dy);
            drawing_area_for_update.queue_draw();
        });

        drawing_area.add_controller(drag);

        // ==== クリック操作（押してる間だけ青） ====
        let click = GestureClick::new();
        let circle_color_for_click = circle_color.clone();
        let drawing_area_for_click = drawing_area.clone();

        click.connect_pressed(move |_, _, _, _| {
            *circle_color_for_click.borrow_mut() = active_color;
            drawing_area_for_click.queue_draw();
        });

        // クリック（ドラッグ）終了時に色を戻す
        let circle_color_for_release = circle_color.clone();
        let drawing_area_for_release = drawing_area.clone();

        click.connect_released(move |_, _, _, _| {
            *circle_color_for_release.borrow_mut() = normal_color;
            drawing_area_for_release.queue_draw();
        });

        drawing_area.add_controller(click);



        // playheadを右クリック中は追従させる
        let drag = GestureDrag::new();
        let drag_offset = Rc::new(RefCell::new((0.0, 0.0)));

        let circle_pos_for_begin = circle_pos.clone();
        let drag_offset_for_begin = drag_offset.clone();

        drag.connect_drag_begin(move |_, start_x, start_y| {
            let (cx, cy) = *circle_pos_for_begin.borrow();
            drag_offset_for_begin.borrow_mut().0 = start_x - cx;
            drag_offset_for_begin.borrow_mut().1 = start_y - cy;
        });

        let circle_pos_for_update = circle_pos.clone();
        let drag_offset_for_update = drag_offset.clone();
        let drawing_area_for_update = drawing_area.clone();

        drag.connect_drag_update(move |_, offset_x, offset_y| {
            let dx = offset_x - drag_offset_for_update.borrow().0;
            let dy = offset_y - drag_offset_for_update.borrow().1;
            *circle_pos_for_update.borrow_mut() = (dx, dy);
            drawing_area_for_update.queue_draw();
        });

        drawing_area.add_controller(drag);

        // ==== クリック操作（押してる間だけ青） ====
        let click = GestureClick::new();
        let circle_color_for_click = circle_color.clone();
        let drawing_area_for_click = drawing_area.clone();

        click.connect_pressed(move |_, _, _, _| {
            *circle_color_for_click.borrow_mut() = active_color;
            drawing_area_for_click.queue_draw();
        });

        // クリック（ドラッグ）終了時に色を戻す
        let circle_color_for_release = circle_color.clone();
        let drawing_area_for_release = drawing_area.clone();

        click.connect_released(move |_, _, _, _| {
            *circle_color_for_release.borrow_mut() = normal_color;
            drawing_area_for_release.queue_draw();
        });

        drawing_area.add_controller(click);





        // ==== ヘッダーバー ====
        let header_box = GtkBox::new(Orientation::Horizontal, 6);
        header_box.set_halign(Align::Start);
        let title_label = Label::new(Some("Luvita"));
        title_label.set_markup("<b>Luvita</b>");
        //let switch = build_switch();
        header_box.append(&title_label);
        //header_box.append(&switch);

        // SVG画像を読み込み
        let logo = Image::from_file("AviUtl_icon.png");
        let save = Image::from_file("save_icon.svg");
        // アイコンサイズ調整（必要なら）
        logo.set_pixel_size(24);
        save.set_pixel_size(20);

        // // 書き出しボタンを作成
        // let export_button = Button::builder()
        //     .label("書き出し")
        //     .build();

        // // 書き出しボタンに画像を設定
        //  let export_icon = Image::from_file("save_icon.svg");
        //  export_icon.set_pixel_size(20); // アイコンのサイズ調整
        //  export_button.set_child(Some(&export_icon)); // 画像をボタンの子要素として設定

        // マージンを追加してロゴを少し右に移動
        //logo.set_margin_start(5);
        save.set_margin_start(60);

        // GestureClickを作成して画像に設定
        //let click_gesture = GestureClick::new();
        //logo.add_controller(click_gesture.clone());

        let header = HeaderBar::builder()
            .title_widget(&header_box)
            .show_end_title_buttons(true)
            .build();

        // ヘッダーに色を付ける
        //header.add_css_class("custom-header");
        // header.pack_end(&export_button); // 書き出しボタンを右端に


        //-------------------------------
        let loader = PixbufLoader::new();
        loader.write(ICON_DATA).unwrap();
        loader.close().unwrap();

        let pixbuf = loader.pixbuf().unwrap();
        let logo = Image::from_pixbuf(Some(&pixbuf));
        logo.set_pixel_size(24); // 必要ならサイズ調整
        //-------------------------------

        // HeaderBarの左端に画像を配置
        //header.pack_start(&logo);
        // 右端にsave iconを配置
        header.pack_start(&save);

        // ==== 表示切替スイッチ ====
        let content_switch = Switch::builder().halign(Align::Start).build();
        content_switch.set_active(true);

        let drawing_area_clone = drawing_area.clone();
        content_switch.connect_active_notify(move |s| {
            drawing_area_clone.set_visible(s.is_active());
        });

        let hbox = GtkBox::new(Orientation::Horizontal, 5);
        hbox.append(&Label::new(Some("表示:")));
        hbox.append(&content_switch);

        // ==== UI組み立て ====
        let vbox = GtkBox::new(Orientation::Vertical, 0);
        vbox.append(&header);
        //vbox.append(&hbox);
        vbox.append(&drawing_area);

        window.set_content(Some(&vbox));

        window.present();
    });

    app.run();
}

// draw_areaの表示、非表示
fn build_switch() -> Switch {
    let switch = Switch::builder().build();
    switch.connect_active_notify(|s| {
        println!("state changed: {:?}", s.is_active());
    });
    switch
}

// CSSの適用（廃止）
// fn apply_custom_css() {
//     let css = "
//         headerbar.custom-header {

//             padding-top: -40px;
//             padding-bottom: -40px;
//             max-height: 12px;
//         }

//         headerbar.custom-header > * {
//             margin-top: 0;
//             margin-bottom: 0;
//         }

//         headerbar.custom-header label {
//             color: white;
//             font-size: 12px;
//             font-weight: bold;
//         }

//     ";

//     let provider = CssProvider::new();
//     provider.load_from_data(css);

//     if let Some(display) = Display::default() {
//         gtk4::style_context_add_provider_for_display(
//             &display,
//             &provider,
//             gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
//         );
//     }
// }
