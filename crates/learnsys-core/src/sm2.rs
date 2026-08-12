//! SM-2 间隔重复算法（移植自 Python `sm2.py`）。
//!
//! 输入当前调度状态 + 本次复习质量分(0–5)，输出新状态。纯函数，无副作用，
//! 不碰数据库——持久化由 [`crate::repo::review_card`] 负责。

use chrono::{Duration, NaiveDate};

#[derive(Debug, Clone, PartialEq)]
pub struct Sm2State {
    pub ef: f64,
    pub interval: i64,
    pub reps: i64,
    pub due: NaiveDate,
}

/// SM-2 核心调度。
///
/// 质量分含义（复习时自评）：
/// - 0–2 完全想不起 → 重置 reps，明天再来
/// - 3   想起但费劲/有错 → 间隔重置为 1
/// - 4   想起，稍迟疑 → 间隔按 EF 增长
/// - 5   瞬间完美 → 间隔按 EF 增长（EF 也升）
pub fn sm2(ef: f64, interval: i64, reps: i64, quality: i64, today: NaiveDate) -> Sm2State {
    let q = quality.clamp(0, 5) as f64;
    // EF 更新：答得越好升得越多，答得差往下掉，地板 1.3
    let new_ef = (ef + (0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02))).max(1.3);

    let (new_reps, new_interval) = if q < 3.0 {
        (0, 1) // 失败：重置
    } else {
        let r = reps + 1;
        let i = if r == 1 {
            1
        } else if r == 2 {
            6
        } else {
            ((interval as f64) * new_ef).round().max(1.0) as i64
        };
        (r, i)
    };

    Sm2State {
        ef: (new_ef * 1000.0).round() / 1000.0,
        interval: new_interval,
        reps: new_reps,
        due: today + Duration::days(new_interval),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn new_card_q5_first_review() {
        let s = sm2(2.5, 0, 0, 5, d("2026-08-12"));
        assert_eq!(s.reps, 1);
        assert_eq!(s.interval, 1);
        assert_eq!(s.due, d("2026-08-13"));
        assert!((s.ef - 2.6).abs() < 1e-9); // 2.5 + 0.1
    }

    #[test]
    fn new_card_q4_keeps_ef() {
        let s = sm2(2.5, 0, 0, 4, d("2026-08-12"));
        assert_eq!(s.reps, 1);
        assert_eq!(s.interval, 1);
        assert!((s.ef - 2.5).abs() < 1e-9);
    }

    #[test]
    fn new_card_q3_drops_ef() {
        let s = sm2(2.5, 0, 0, 3, d("2026-08-12"));
        assert_eq!(s.reps, 1);
        assert_eq!(s.interval, 1);
        assert!((s.ef - 2.36).abs() < 1e-9); // 2.5 - 0.14
    }

    #[test]
    fn second_review_q5_jumps_to_6() {
        // reps=1 interval=1 ef=2.6
        let s = sm2(2.6, 1, 1, 5, d("2026-08-13"));
        assert_eq!(s.reps, 2);
        assert_eq!(s.interval, 6);
        assert_eq!(s.due, d("2026-08-19"));
        assert!((s.ef - 2.7).abs() < 1e-9);
    }

    #[test]
    fn third_review_uses_ef() {
        // reps=2 interval=6 ef=2.7 → ef=2.8, interval=round(6*2.8)=17
        let s = sm2(2.7, 6, 2, 5, d("2026-08-19"));
        assert_eq!(s.reps, 3);
        assert_eq!(s.interval, 17);
    }

    #[test]
    fn q2_resets_reps() {
        let s = sm2(2.5, 10, 5, 2, d("2026-08-12"));
        assert_eq!(s.reps, 0);
        assert_eq!(s.interval, 1);
        assert_eq!(s.due, d("2026-08-13"));
    }

    #[test]
    fn quality_is_clamped() {
        let s = sm2(2.5, 0, 0, 9, d("2026-08-12")); // 9 → 5
        assert_eq!(s.reps, 1);
        assert!((s.ef - 2.6).abs() < 1e-9);
    }

    #[test]
    fn ef_floor_is_130() {
        // 反复答差，EF 不会跌破 1.3
        let mut ef = 1.4;
        for _ in 0..5 {
            ef = sm2(ef, 1, 0, 0, d("2026-08-12")).ef;
        }
        assert!(ef >= 1.3);
    }
}
