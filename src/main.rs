// Для удобства импортируем всё их macroquad.
use macroquad::prelude::*;

// Делаем main асинхронной, задавая, заодно, заголовок окна.
// Асинхронность требуется, в основном, для лучшей совместимости с WASM.
#[macroquad::main("Snake")]
async fn main() {
    let mut snake = Snake::default();
    let mut fruit = Fruit::respawn();

    // Игровой цикл
    loop {
        // Очищаем экран, заполняя его серым цветом.
        clear_background(LIGHTGRAY);

        // Перемещаем змейку.
        let dt = get_frame_time();

        // В зависимости от нажатых клавиш, меняем направление движения змейки.
        let mut rotation = 0.0;
        if is_key_down(KeyCode::Left) {
            rotation += ROTATION_PER_SEC;
        }
        if is_key_down(KeyCode::Right) {
            rotation -= ROTATION_PER_SEC;
        }
        snake.go(dt, rotation);

        // Если змейка съела фрукт, то увеличиваем ее длину и создаем новый фрукт.
        if snake.can_eat(&fruit) {
            fruit = Fruit::respawn();
            snake.add_unit();
        }

        // Если игрок потерпел поражение, начинаем игру заново:
        if snake.is_lose() {
            snake = Snake::default();
            fruit = Fruit::respawn();
        }

        draw_field();
        snake.draw();
        fruit.draw(); // Отображаем фрукт.

        // Отображаем количество очков: длина змейки, не считая головы.
        let scores_text = format!("Scores: {}", snake.length() - 1);
        draw_text(&scores_text, 20.0, 20.0, 24.0, BLACK);

        // Дожидаемся следующего кадра.
        // Это нужно, чтобы FPS был стабилен и равен 60.
        next_frame().await
    }
}

// Задаём параметры. Размеры будем задавать в метрах, а углы в радианах.

/// Размер игрового поля.
const FIELD_SIZE: f32 = 2.0;

/// Начальная скорость змеи.
const INIT_SPEED: f32 = 0.4;

/// Радиус элемента змеи.
const UNIT_RADIUS: f32 = 0.04;

/// Радиус фрукта.
const FRUIT_RADIUS: f32 = 0.06;

/// Скорасть вращения змейки (радианы в секунду).
const ROTATION_PER_SEC: f32 = 2.0;

/// Для рисования нам потребуются размеры в пикселях.
/// Вычисляем их по меньшей стороне окна.
fn pixels_per_meter() -> f32 {
    screen_width().min(screen_height()) / 2.0
}

/// Переводим координаты точки на игровом поле в координаты окна.
fn to_screen_coords(pos: Vec2) -> Vec2 {
    // Так как поле будет квадратным, а окно может быть прямоугольным,
    // вычиляем отступ от края, для большей стороны окна.
    let min_dim = screen_width().min(screen_height());
    let width_offset = (screen_width() - min_dim) / 2.0;
    let height_offset = (screen_height() - min_dim) / 2.0;
    let offset = Vec2::new(width_offset, height_offset);

    // Переводим координаты игрового поля в координаты окна.
    let shift = Vec2::new(1.0, -1.0); // Смещение центра координат.
    let scale = Vec2::new(1.0, -1.0) * pixels_per_meter(); // Масштаб.
    (pos + shift) * scale + offset
}

/// Рисуем игровое поле.
fn draw_field() {
    // Координаты верхнего левого угла поля в пикселях.
    let top_left = to_screen_coords(Vec2::new(-1.0, 1.0));
    // Размер поля в пикселях.
    let size = pixels_per_meter() * FIELD_SIZE * Vec2::ONE;

    // Рисуем поле в виде зелёного прямоугольника.
    draw_rectangle(top_left.x, top_left.y, size.x, size.y, GREEN);
}

/// Элемент змейки
#[derive(Clone, Copy)]
struct Unit {
    position: Vec2,
}

impl Unit {
    /// Элемент будет перемещаться в сторону предыдущего элемента змейки, если тот отдаляется.
    pub fn go(&mut self, prev_unit_pos: Vec2) {
        let to_prev = prev_unit_pos - self.position;
        let distance = to_prev.length();
        let shift = distance - 2.0 * UNIT_RADIUS;

        // Если расстояние до следующего элемента больше двух радиусов,
        // то смещаемся, чтобы змейка не разрывалась.
        if shift > 0.0 {
            self.position += to_prev.normalize() * shift;
        }
    }

    /// Отображение элемента змейки в виде белого круга.
    pub fn draw(&self) {
        let radius_pixels = UNIT_RADIUS * pixels_per_meter();
        let screen_pos = to_screen_coords(self.position);
        draw_circle(screen_pos.x, screen_pos.y, radius_pixels, WHITE);
    }

    pub fn intersect(&self, position: Vec2, radius: f32) -> bool {
        self.position.distance(position) < radius + UNIT_RADIUS
    }
}

/// Голова змейки.
/// Это особый элемент змейки, который вращается и двигается согласно действиям пользователя.
struct Head {
    unit: Unit,
    direction: Vec2,
    speed: f32,
}

impl Head {
    /// Вращение головы змейки на указанный угол.
    pub fn rotate(&mut self, angle: f32) {
        let rotation = Vec2::from_angle(angle);
        let new_head_direction = rotation.rotate(self.direction);
        self.direction = new_head_direction;
    }

    /// Перемещение головы змейки.
    pub fn go(&mut self, dt: f32) {
        self.unit.position += self.speed * dt * self.direction;
    }

    /// Возвращаем позицию головы.
    pub fn position(&self) -> Vec2 {
        self.unit.position
    }

    /// Отображаем голову змейки.
    fn draw(&self) {
        self.unit.draw();

        // Помимо отображения обычного сегмента, отобразим глаза по направлению движекния.
        let angle = 0.3; // Половина угла между глазами.
        let left_eye_shift = Vec2::from_angle(angle).rotate(self.direction) * UNIT_RADIUS;
        let left_eye_pos = to_screen_coords(self.position() + left_eye_shift);
        let right_eye_shift = Vec2::from_angle(-angle).rotate(self.direction) * UNIT_RADIUS;
        let right_eye_pos = to_screen_coords(self.position() + right_eye_shift);
        let eye_r = UNIT_RADIUS / 6.0 * pixels_per_meter();

        draw_circle(left_eye_pos.x, left_eye_pos.y, eye_r, BLACK);
        draw_circle(right_eye_pos.x, right_eye_pos.y, eye_r, BLACK);
    }

    pub fn intersect(&self, position: Vec2, radius: f32) -> bool {
        self.unit.intersect(position, radius)
    }
}

/// Змейка - это голова и сегменты.
struct Snake {
    head: Head,
    units: Vec<Unit>,
}

impl Snake {
    /// Перемещение змейки - это вращение и перемещение головы и, затем, последовательное перемещение всех сегментов.
    pub fn go(&mut self, dt: f32, rotation: f32) {
        let angle = rotation * dt;
        self.head.rotate(angle);
        self.head.go(dt);

        let mut prev_unit_pos = self.head.position();
        for unit in &mut self.units {
            unit.go(prev_unit_pos);
            prev_unit_pos = unit.position;
        }
    }

    /// Отображение змейки.
    pub fn draw(&self) {
        self.head.draw();
        for unit in &self.units {
            unit.draw();
        }
    }

    /// Длина змейки.
    pub fn length(&self) -> u32 {
        (self.units.len() + 1) as _
    }

    // Если голова змейки пересикается с фруктом, то она может его съесть.
    pub fn can_eat(&self, fruit: &Fruit) -> bool {
        self.head.intersect(fruit.position, FRUIT_RADIUS)
    }

    /// Добавляем новый сегмент к змейке.
    pub fn add_unit(&mut self) {
        // В качестве позиции используем позицию последнего сегмента.
        // Или головы, если сегментов нет.
        let last_unit = self.units.last().unwrap_or(&self.head.unit);
        self.units.push(*last_unit);
    }

    /// Проверка на поражение.
    pub fn is_lose(&self) -> bool {
        // Либо при пересечении с сегментом.
        let intersect_unit = self
            .units
            .iter()
            .skip(1) // пропускаем проверку пересечения с сегментом, соединённым с головой.
            .any(|u| self.head.intersect(u.position, UNIT_RADIUS * 0.8));

        // Либо при пересечении с границами поля.
        let max_coord = FIELD_SIZE / 2.0 - UNIT_RADIUS;
        let intersect_wall =
            self.head.position().x.abs() > max_coord || self.head.position().y.abs() > max_coord;

        intersect_unit || intersect_wall
    }
}

/// По умолчанию у змейки есть только голова.
impl Default for Snake {
    fn default() -> Self {
        let head_unit = Unit {
            position: Vec2::ZERO,
        };

        let head = Head {
            unit: head_unit,
            direction: Vec2::X,
            speed: INIT_SPEED,
        };

        Self {
            head,
            units: vec![],
        }
    }
}



/// Возвращает случайное число от 0.0 до 1.0
fn rand_f32() -> f32 {
    (rand::rand() as f64 / u32::MAX as f64) as f32
}

/// Возвращает случайную позицию в игровом поле.
fn random_position() -> Vec2 {
    Vec2::new(rand_f32() * 2.0 - 1.0, rand_f32() * 2.0 - 1.0)
}


/// Фрукт, который можно собрать.
struct Fruit {
    position: Vec2,
}

impl Fruit {
    /// Фрукт будет появляться в случайном месте игрового поля.
    pub fn respawn() -> Self {
        Self {
            position: random_position(),
        }
    }

    /// Фрукт будет отображаться в виде красного круга.
    pub fn draw(&self) {
        let ppm = pixels_per_meter();
        let radius_pixels = FRUIT_RADIUS * ppm;
        let screen_pos = to_screen_coords(self.position);
        draw_circle(screen_pos.x, screen_pos.y, radius_pixels, RED);
    }
}

