use chinchilib::rgb::RGBA8;
use std::f32::consts::PI;

pub trait Lerp {
    fn lerp(start: Self, end: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(start: Self, end: Self, t: f32) -> Self {
        start + (end - start) * t
    }
}

impl Lerp for RGBA8 {
    fn lerp(start: Self, end: Self, t: f32) -> Self {
        let r = (start.r as f32 + (end.r as f32 - start.r as f32) * t) as u8;
        let g = (start.g as f32 + (end.g as f32 - start.g as f32) * t) as u8;
        let b = (start.b as f32 + (end.b as f32 - start.b as f32) * t) as u8;
        let a = (start.a as f32 + (end.a as f32 - start.a as f32) * t) as u8;
        RGBA8 { r, g, b, a }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EasingStyle {
    Sine,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EasingDirection {
    In,
    Out,
    InOut,
}

#[derive(Debug, Clone, Copy)]
pub struct TweenInfo {
    /// Time of the tween execution
    pub time: f32,
    /// Easing Style (lerp function)
    pub style: EasingStyle,
    /// Easing Direction (lerp direction)
    pub direction: EasingDirection,
}

impl Default for TweenInfo {
    fn default() -> Self {
        Self {
            time: 1.0,
            style: EasingStyle::Sine,
            direction: EasingDirection::Out,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tween<T> {
    /// Value at start point (before tweening)
    start_value: T,
    /// Value where the tweening goes
    end_value: T,
    /// Duration of the tweening
    duration: f32,
    /// Time elapsed since the begining of the tween
    elapsed: f32,
    /// Easing Style (lerp function)
    style: EasingStyle,
    /// Easing Direction (lerp direction)
    direction: EasingDirection,
    /// Is the tweening effective ?
    pub active: bool,
}

impl<T: Lerp + Copy> Tween<T> {
    pub fn new(start: T, end: T, info: TweenInfo) -> Self {
        Self {
            start_value: start,
            end_value: end,
            duration: info.time.max(0.001),
            elapsed: 0.0,
            style: info.style,
            direction: info.direction,
            active: true,
        }
    }

    pub fn update(&mut self, dt: f32) -> T {
        if !self.active {
            return self.end_value;
        }
        self.elapsed += dt;
        if self.elapsed >= self.duration {
            self.elapsed = self.duration;
            self.active = false;
            return self.end_value;
        }
        let t = self.elapsed / self.duration;
        let eased_t = self.apply_easing(t);
        T::lerp(self.start_value, self.end_value, eased_t)
    }

    fn apply_easing(&self, t: f32) -> f32 {
        match self.style {
            EasingStyle::Linear => t,
            EasingStyle::Sine => match self.direction {
                EasingDirection::In => 1.0 - (t * PI / 2.0).cos(),
                EasingDirection::Out => (t * PI / 2.0).sin(),
                EasingDirection::InOut => -((t * PI).cos() - 1.0) / 2.0,
            },
        }
    }
}

pub struct TweenService;

impl TweenService {
    pub fn create<T: Lerp + Copy>(start: T, end: T, info: TweenInfo) -> Tween<T> {
        Tween::new(start, end, info)
    }
}
