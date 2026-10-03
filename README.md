# slide-captcha-rs

Rust 滑块验证码库，支持滑动与拖拽。

## 安装

```toml
[dependencies]
slide-captcha-rs = "0.1"
```

## 特性

- **Basic**：水平滑块，拼图块与缺口同一 Y 轴
- **DragDrop**：从死区边缘拖拽拼图块到缺口
- **程序化拼图**：未提供自定义图形时，自动生成圆角矩形 + 右侧凸起的滑块块
- **自定义图形**：可传入 overlay / shadow / mask 三张图

## 用法

```rust
use slide_captcha_rs::{
    validate, with_backgrounds, with_graph_images, with_image_size, Builder, GraphImage, Size,
};

let captcha = Builder::new()
    .set_options(with_image_size(Size {
        width: 300,
        height: 220,
    }))
    .set_resources(with_backgrounds(vec![bg]))
    // 可选：.set_resources(with_graph_images(vec![graph]))
    .make(); // 或 .make_drag_drop()

let data = captcha.generate()?;
let block = data.block();
let master_jpeg: Vec<u8> = data.master_image().to_bytes()?; // JPEG
let tile_png: Vec<u8> = data.tile_image().to_bytes()?; // PNG
let b64 = data.master_image().to_base64()?; // data:image/jpeg;base64,...

let ok = validate(user_x, user_y, block.x, block.y, 5);
```

自定义图形：

```rust
let graph = GraphImage::new(overlay, shadow, mask);
let captcha = Builder::new()
    .set_resources([
        with_backgrounds(vec![bg]),
        with_graph_images(vec![graph]),
    ])
    .make();
```

`EncodedImage` 方法：`to_bytes()`、`to_base64()`（带 data-URI 前缀）、`to_base64_data()`（纯 Base64）、`save(path)`。

主图默认 JPEG 质量 80；拼图块为 PNG（含 alpha）。

## 配置

| 选项 | 默认值 | 说明 |
| --- | --- | --- |
| `with_image_size` | 300×220 | 主图尺寸 |
| `with_image_alpha` | 1.0 | 图像透明度 |
| `with_range_graph_size` | 60..=70 | 拼图块边长范围 |
| `with_range_graph_angle_pos` | 0..=0 | 旋转角范围（v0.1 不旋转） |
| `with_gen_graph_number` | 1 | 主图上绘制的缺口数量 |
| `with_enable_graph_vertical_random` | `false` | 多缺口时是否各自随机 Y |
| `with_range_dead_zone_directions` | 四边 | 死区方向；Basic 模式强制为 Left |

资源：

| 资源 | 必填 | 说明 |
| --- | --- | --- |
| `with_backgrounds` | 是 | 至少一张背景图 |
| `with_graph_images` | 否 | 自定义 overlay / shadow / mask |

## 校验

```rust
fn validate(sx: i32, sy: i32, dx: i32, dy: i32, padding: i32) -> bool
```

判定用户提交点 `(sx, sy)` 是否落在目标点 `(dx, dy)` 的 `padding` 容差内（两轴均为闭区间 `[d - padding, d + padding]`）。

## 示例

```bash
cargo run --example generate
```

在 `temp/master.jpg` 与 `temp/tile.png` 写出一组滑块图。

## 说明

算法与交互模型参考 [go-captcha](https://github.com/wenlng/go-captcha)，本仓库为独立 Rust 实现。

## License

Apache-2.0，Copyright 2026 kk
