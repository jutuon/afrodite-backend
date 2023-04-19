//! Index for profiles
//!
//! LocationIndex
//!
//! Idea is to make matrix which has up-down lookup with atomic u16 values.
//! Those atomic values represents matrix indexes.
//!
//! Perhaps left-right lookup could be implemented as well??
//! Yes, it should be possible. Then there will be for atomic values in one cell.
//! Figure out first the up-down lookup.
//!
//! Best to use u16 for atomic numbers, so algorithm will be easier.
//! Matrix index numbers will fit to u16.
//!
//! Matrix cell should contain boolean which represents is there some profile in it.
//!
//! Initialization should happen so that border values of matrix should be used.
//!
//! Only one writer allowed at one time.
//!
//! No locks needed.
//!
//! Matrix indexes are used like a key for HashMap<(u16,u16), Vec<AccountId>>

use std::{
    num::NonZeroU16,
    ops::Index,
    sync::{
        atomic::{AtomicBool, AtomicI16, AtomicU16, Ordering},
        Arc,
    },
};

use nalgebra::{DMatrix, Dyn, VecStorage};

use crate::api::account::data;

#[derive(Debug, Hash, PartialEq, Clone, Copy)]
pub struct LocationIndexKey {
    pub y: u16,
    pub x: u16,
}

impl LocationIndexKey {
    fn x(&self) -> usize {
        self.x as usize
    }

    fn y(&self) -> usize {
        self.y as usize
    }
}

impl Index<LocationIndexKey> for DMatrix<CellData> {
    type Output = <Self as Index<(usize, usize)>>::Output;

    fn index(&self, key: LocationIndexKey) -> &Self::Output {
        &self[(key.y as usize, key.x as usize)]
    }
}

/// Origin (0,0) = (y, x) is at top left corner.
pub struct LocationIndex {
    //data1: Box<[[CellData; HEIGHT]; WIDTH]>,
    data: DMatrix<CellData>,
    width: u16,
    height: u16,
}

impl LocationIndex {
    pub fn new(width: NonZeroU16, height: NonZeroU16) -> Self {
        let size = (width.get() as usize) * (height.get() as usize);
        let mut data = Vec::with_capacity(size);
        data.resize_with(size, || CellData::new(width.get(), height.get()));
        let storage = VecStorage::new(Dyn(height.get() as usize), Dyn(width.get() as usize), data);
        Self {
            data: DMatrix::from_data(storage),
            width: width.get(),
            height: height.get(),
        }
    }

    pub fn data(&self) -> &DMatrix<CellData> {
        &self.data
    }

    /// Index width. Greater than zero.
    pub fn width(&self) -> usize {
        self.width as usize
    }

    /// Index height. Greater than zero.
    pub fn height(&self) -> usize {
        self.height as usize
    }

    pub fn last_row_index(&self) -> usize {
        self.height() - 1
    }

    pub fn last_column_index(&self) -> usize {
        self.width() - 1
    }
}

#[derive(Debug)]
pub struct CellData {
    pub next_up: AtomicU16,
    pub next_down: AtomicU16,
    pub next_left: AtomicU16,
    pub next_right: AtomicU16,
    pub profiles_in_this_area: AtomicBool,
}

impl CellData {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            next_down: AtomicU16::new(height.checked_sub(1).unwrap()),
            next_up: AtomicU16::new(0),
            next_left: AtomicU16::new(0),
            next_right: AtomicU16::new(width.checked_sub(1).unwrap()),
            profiles_in_this_area: AtomicBool::new(false),
        }
    }

    pub fn next_down(&self) -> usize {
        self.next_down.load(Ordering::Relaxed) as usize
    }

    pub fn next_up(&self) -> usize {
        self.next_up.load(Ordering::Relaxed) as usize
    }

    pub fn next_left(&self) -> usize {
        self.next_left.load(Ordering::Relaxed) as usize
    }

    pub fn next_right(&self) -> usize {
        self.next_right.load(Ordering::Relaxed) as usize
    }

    pub fn profiles(&self) -> bool {
        self.profiles_in_this_area.load(Ordering::Relaxed)
    }

    pub fn set_next_down(&self, i: usize) {
        self.next_down.store(i as u16, Ordering::Relaxed)
    }

    pub fn set_next_up(&self, i: usize) {
        self.next_up.store(i as u16, Ordering::Relaxed)
    }

    pub fn set_next_left(&self, i: usize) {
        self.next_left.store(i as u16, Ordering::Relaxed)
    }

    pub fn set_next_right(&self, i: usize) {
        self.next_right.store(i as u16, Ordering::Relaxed)
    }

    pub fn set_profiles(&self, value: bool) {
        self.profiles_in_this_area.store(value, Ordering::Relaxed)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
struct VisitedMaxCorners {
    pub top_left: bool,
    pub top_right: bool,
    pub bottom_left: bool,
    pub bottom_right: bool,
}

impl VisitedMaxCorners {
    fn all_visited(&self) -> bool {
        self.bottom_left && self.bottom_right && self.top_left && self.top_right
    }
}

/// Iterator for location index
///
/// Start from one cell and enlarge area clockwise.
/// Each iteration starts from one cell down of top right corner.
/// Iteration ends to top right corner.
pub struct LocationIndexIterator {
    index: Arc<LocationIndex>,
    init_position_y: isize,
    init_position_x: isize,
    x: isize,
    y: isize,
    /// How many rounds cursor has moved. Checking initial position counts one.
    iteration_count: isize,
    iter_init_position_x: isize,
    iter_init_position_y: isize,
    /// Move direction for cursor
    direction: Direction,
    /// No more new cells available.
    completed: bool,
    visited_max_corners: VisitedMaxCorners,
}

impl LocationIndexIterator {
    pub fn new(index: Arc<LocationIndex>) -> Self {
        Self {
            index,
            y: 0,
            x: 0,
            init_position_y: 0,
            init_position_x: 0,
            iteration_count: 0,
            iter_init_position_x: 0,
            iter_init_position_y: 0,
            direction: Direction::Down,
            completed: false,
            visited_max_corners: VisitedMaxCorners::default(),
        }
    }

    pub fn reset(&mut self, x: u16, y: u16) {
        let x = x as isize;
        let y = y as isize;
        self.x = x.min(self.index.width() as isize - 1);
        self.y = y.min(self.index.height() as isize - 1);
        self.init_position_x = self.x;
        self.init_position_y = self.y;
        self.iter_init_position_x = self.x;
        self.iter_init_position_y = self.y;
        self.iteration_count = 0;
        self.direction = Direction::Down;
        self.completed = false;
        self.visited_max_corners = VisitedMaxCorners::default();
    }

    /// Get next cell where are profiles.
    pub fn next(&mut self) -> Option<LocationIndexKey> {
        self.next_raw().map(|(y, x)| LocationIndexKey { y, x })
    }

    /// Get next cell where are profiles.
    ///
    /// Returns key for HashMap. Key is (y, x)
    fn next_raw(&mut self) -> Option<(u16, u16)> {
        if self.completed {
            return None;
        }

        loop {
            let data_position = if self.current_cell_has_profiles() {
                Some((self.y as u16, self.x as u16))
            } else {
                None
            };

            match self.move_next_position() {
                Ok(()) => (),
                Err(()) => {
                    self.completed = true;
                    return data_position;
                }
            }

            if data_position.is_some() {
                return data_position;
            }
        }
    }

    /// Left side max area index.
    fn current_left_max_index(&self) -> isize {
        self.init_position_x - self.iteration_count
    }

    /// Right side max area index.
    fn current_right_max_index(&self) -> isize {
        self.init_position_x + self.iteration_count
    }

    /// Top side max area index.
    fn current_top_max_index(&self) -> isize {
        self.init_position_y - self.iteration_count
    }

    /// Bottom side max area index.
    fn current_bottom_max_index(&self) -> isize {
        self.init_position_y + self.iteration_count
    }

    fn current_cell_has_profiles(&self) -> bool {
        self.current_cell()
            .map(|cell| cell.profiles())
            .unwrap_or(false)
    }

    fn current_cell(&self) -> Option<&CellData> {
        if self.y < 0 || self.y >= self.index.height() as isize {
            return None;
        }
        if self.x < 0 || self.x >= self.index.width() as isize {
            return None;
        }

        Some(&self.index.data()[(self.y as usize, self.x as usize)])
    }

    /// Move position according to cell next index information.
    ///
    /// Returns error if there is no next new position.
    fn move_next_position(&mut self) -> Result<(), ()> {
        if self.visited_max_corners.all_visited() && self.current_round_complete() {
            return Err(());
        }

        if self.current_round_complete() {
            self.move_to_next_round_init_pos();
            self.update_visited_max_corners();
            return Ok(());
        }

        // Make move
        match self.direction {
            Direction::Up => {
                if self.y >= self.index.height() as isize {
                    // Bottom: outside matrix
                    self.y = self.index.last_row_index() as isize;
                } else if self.y <= 0 {
                    // Top: top line or outside matrix
                    self.y = self.current_top_max_index();
                } else {
                    // Normal: inside matrix area and not the first row.
                    self.y = self
                        .current_cell()
                        .map_or(0, |c| c.next_up() as isize)
                        .max(self.current_top_max_index())
                }
            }
            Direction::Down => {
                if self.y >= self.index.last_row_index() as isize {
                    // Bottom: outside matrix or bottom row
                    self.y = self.current_bottom_max_index();
                } else if self.y < 0 {
                    // Top: top line or outside matrix
                    self.y = 0;
                } else {
                    // Normal: inside matrix area and not the last row.
                    self.y = self
                        .current_cell()
                        .map_or(self.index.last_row_index() as isize, |c| {
                            c.next_down() as isize
                        })
                        .min(self.current_bottom_max_index())
                }
            }
            Direction::Left => {
                if self.x > self.index.last_column_index() as isize {
                    // Right: outside matrix
                    self.x = self.index.last_column_index() as isize;
                } else if self.x <= 0 {
                    // Left: left column or outside matrix
                    self.x = self.current_left_max_index();
                } else {
                    // Normal: inside matrix area and not the left column.
                    self.x = self
                        .current_cell()
                        .map_or(0 as isize, |c| c.next_left() as isize)
                        .max(self.current_left_max_index())
                }
            }
            Direction::Right => {
                if self.x >= self.index.last_column_index() as isize {
                    // Right: outside matrix or last column
                    self.x = self.current_right_max_index();
                } else if self.x < 0 {
                    // Left: outside matrix
                    self.x = 0;
                } else {
                    // Normal: inside matrix area and not the right column.
                    self.x = self
                        .current_cell()
                        .map_or(self.index.last_column_index() as isize, |c| {
                            c.next_right() as isize
                        })
                        .min(self.current_right_max_index())
                }
            }
        }

        // Change direction if needed
        if self.x == self.current_right_max_index() && self.y == self.current_top_max_index() {
            self.direction = Direction::Down;
        } else if self.x == self.current_right_max_index()
            && self.y == self.current_bottom_max_index()
        {
            self.direction = Direction::Left;
        } else if self.x == self.current_left_max_index()
            && self.y == self.current_bottom_max_index()
        {
            self.direction = Direction::Up;
        } else if self.x == self.current_left_max_index() && self.y == self.current_top_max_index()
        {
            self.direction = Direction::Right;
        }

        self.update_visited_max_corners();

        Ok(())
    }

    fn current_round_complete(&self) -> bool {
        self.iter_init_position_x == self.x
            && self.iter_init_position_y == self.y
            && self.direction == Direction::Down
    }

    /// Top left corner starts the game
    fn move_to_next_round_init_pos(&mut self) {
        self.iteration_count += 1;
        self.direction = Direction::Down;
        self.visited_max_corners = VisitedMaxCorners::default();
        self.x = self.current_right_max_index();
        self.y = self.current_top_max_index();
        self.iter_init_position_x = self.x;
        self.iter_init_position_y = self.y;

        // Move to next than the iter init position
        self.y += 1;
    }

    fn update_visited_max_corners(&mut self) {
        if self.y <= 0 && self.x <= 0 {
            self.visited_max_corners.top_left = true;
        }
        if self.y <= 0 && self.x >= self.index.width() as isize {
            self.visited_max_corners.top_right = true;
        }
        if self.y >= self.index.height() as isize && self.x <= 0 {
            self.visited_max_corners.bottom_left = true;
        }
        if self.y >= self.index.height() as isize && self.x >= self.index.width() as isize {
            self.visited_max_corners.bottom_right = true;
        }
    }
}

/// Update index.
///
/// Create only one IndexUpdater as it modifies the LocationIndex.
pub struct IndexUpdater {
    index: Arc<LocationIndex>,
}

impl IndexUpdater {
    pub fn new(index: Arc<LocationIndex>) -> Self {
        Self { index }
    }

    pub fn flag_cell_to_have_profiles(&mut self, key: LocationIndexKey) {
        if self.index.data[key].profiles() {
            return;
        }

        self.index.data[key].set_profiles(true);

        // Update right side of row
        for c in self.index.data.row(key.y()).iter().skip(key.x() + 1) {
            c.set_next_left(key.x());

            if c.profiles() {
                break;
            }
        }

        // Update left side of row
        for c in self
            .index
            .data
            .row(key.y())
            .iter()
            .rev()
            .skip(self.index.width() - key.x())
        {
            c.set_next_right(key.x());

            if c.profiles() {
                break;
            }
        }

        // Update bottom side of column
        for c in self.index.data.column(key.x()).iter().skip(key.y() + 1) {
            c.set_next_up(key.y());

            if c.profiles() {
                break;
            }
        }

        // Update top side of column
        for c in self
            .index
            .data
            .column(key.x())
            .iter()
            .rev()
            .skip(self.index.height() - key.y())
        {
            c.set_next_down(key.y());

            if c.profiles() {
                break;
            }
        }
    }

    pub fn remove_profile_flag_from_cell(&mut self, key: LocationIndexKey) {
        if !self.index.data[key].profiles() {
            return;
        }

        let cell = &self.index.data[key];
        cell.set_profiles(false);

        let next_right = cell.next_right();
        let next_left = cell.next_left();
        let next_up = cell.next_up();
        let next_down = cell.next_down();

        // Update right side of row
        for c in self.index.data.row(key.y()).iter().skip(key.x() + 1) {
            c.set_next_left(next_left);

            if c.profiles() {
                break;
            }
        }

        // Update left side of row
        for c in self
            .index
            .data
            .row(key.y())
            .iter()
            .rev()
            .skip(self.index.width() - key.x())
        {
            c.set_next_right(next_right);

            if c.profiles() {
                break;
            }
        }

        // Update bottom side of column
        for c in self.index.data.column(key.x()).iter().skip(key.y() + 1) {
            c.set_next_up(next_up);

            if c.profiles() {
                break;
            }
        }

        // Update top side of column
        for c in self
            .index
            .data
            .column(key.x())
            .iter()
            .rev()
            .skip(self.index.height() - key.y())
        {
            c.set_next_down(next_down);

            if c.profiles() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index() -> LocationIndex {
        let index = LocationIndex::new(5.try_into().unwrap(), 10.try_into().unwrap());
        index.data[(0, 0)].set_profiles(true);
        index.data[(0, 4)].set_profiles(true);
        index.data[(9, 0)].set_profiles(true);
        index.data[(9, 4)].set_profiles(true);
        index
    }

    fn mirror_index() -> LocationIndex {
        let index = LocationIndex::new(10.try_into().unwrap(), 5.try_into().unwrap());
        index.data[(0, 0)].set_profiles(true);
        index.data[(0, 9)].set_profiles(true);
        index.data[(4, 0)].set_profiles(true);
        index.data[(4, 9)].set_profiles(true);
        index
    }

    #[test]
    fn top_left_initial_values() {
        assert!(index().data()[(0, 0)].next_up() == 0);
        assert!(index().data()[(0, 0)].next_down() == 9);
        assert!(index().data()[(0, 0)].next_left() == 0);
        assert!(index().data()[(0, 0)].next_right() == 4);
    }

    #[test]
    fn top_right_initial_values() {
        assert!(index().data()[(0, 4)].next_up() == 0);
        assert!(index().data()[(0, 4)].next_down() == 9);
        assert!(index().data()[(0, 4)].next_left() == 0);
        assert!(index().data()[(0, 4)].next_right() == 4);
    }

    #[test]
    fn bottom_left_initial_values() {
        assert!(index().data()[(9, 0)].next_up() == 0);
        assert!(index().data()[(9, 0)].next_down() == 9);
        assert!(index().data()[(9, 0)].next_left() == 0);
        assert!(index().data()[(9, 0)].next_right() == 4);
    }

    #[test]
    fn bottom_right_initial_values() {
        assert!(index().data()[(9, 4)].next_up() == 0);
        assert!(index().data()[(9, 4)].next_down() == 9);
        assert!(index().data()[(9, 4)].next_left() == 0);
        assert!(index().data()[(9, 4)].next_right() == 4);
    }

    #[test]
    fn iterator_top_left_works() {
        let mut iter = LocationIndexIterator::new(index().into());

        let n = iter.next_raw();
        assert!(n == Some((0, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 4)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((9, 4)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((9, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == None, "was: {n:?}");
    }

    #[test]
    fn iterator_top_right_works() {
        let mut iter = LocationIndexIterator::new(index().into());
        iter.reset(4, 0);

        let n = iter.next_raw();
        assert!(n == Some((0, 4)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((9, 4)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((9, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == None, "was: {n:?}");
    }

    #[test]
    fn iterator_bottom_right_works() {
        let mut iter = LocationIndexIterator::new(index().into());
        iter.reset(4, 9);

        let n = iter.next_raw();
        assert!(n == Some((9, 4)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((9, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 4)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == None, "was: {n:?}");
    }

    #[test]
    fn iterator_bottom_left_works() {
        let mut iter = LocationIndexIterator::new(index().into());
        iter.reset(0, 9);

        let n = iter.next_raw();
        assert!(n == Some((9, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((9, 4)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 4)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == None, "was: {n:?}");
    }

    #[test]
    fn mirror_iterator_top_left_works() {
        let mut iter = LocationIndexIterator::new(mirror_index().into());

        let n = iter.next_raw();
        assert!(n == Some((0, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((4, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 9)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((4, 9)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == None, "was: {n:?}");
    }

    #[test]
    fn mirror_iterator_top_right_works() {
        let mut iter = LocationIndexIterator::new(mirror_index().into());
        iter.reset(9, 0);

        let n = iter.next_raw();
        assert!(n == Some((0, 9)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((4, 9)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((4, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == None, "was: {n:?}");
    }

    #[test]
    fn mirror_iterator_bottom_right_works() {
        let mut iter = LocationIndexIterator::new(mirror_index().into());
        iter.reset(9, 4);

        let n = iter.next_raw();
        assert!(n == Some((4, 9)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 9)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((4, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == None, "was: {n:?}");
    }

    #[test]
    fn mirror_iterator_bottom_left_works() {
        let mut iter = LocationIndexIterator::new(mirror_index().into());
        iter.reset(0, 4);

        let n = iter.next_raw();
        assert!(n == Some((4, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 9)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((4, 9)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == None, "was: {n:?}");
    }

    // IndexUpdater

    fn index_for_updater() -> LocationIndex {
        LocationIndex::new(3.try_into().unwrap(), 3.try_into().unwrap())
    }

    #[test]
    fn simple_index_update() {
        let index: Arc<_> = index_for_updater().into();
        let mut updater = IndexUpdater::new(index.clone());
        updater.flag_cell_to_have_profiles(LocationIndexKey { x: 1, y: 1 });

        let test_cell = |key: (usize, usize), up: usize, down: usize, left: usize, right: usize| {
            assert!(index.data[key].next_up() == up);
            assert!(index.data[key].next_down() == down);
            assert!(index.data[key].next_left() == left);
            assert!(index.data[key].next_right() == right);
        };

        // Check middle column
        test_cell((0, 1), 0, 1, 0, 2);
        test_cell((1, 1), 0, 2, 0, 2);
        test_cell((2, 1), 1, 2, 0, 2);

        // Check middle row
        test_cell((1, 0), 0, 2, 0, 1);
        test_cell((1, 1), 0, 2, 0, 2);
        test_cell((1, 2), 0, 2, 1, 2);
    }

    #[test]
    fn simple_index_remove_test() {
        let index: Arc<_> = index_for_updater().into();
        let mut updater = IndexUpdater::new(index.clone());
        updater.flag_cell_to_have_profiles(LocationIndexKey { x: 1, y: 1 });
        updater.remove_profile_flag_from_cell(LocationIndexKey { x: 1, y: 1 });

        let test_cell = |key: (usize, usize), up: usize, down: usize, left: usize, right: usize| {
            assert!(index.data[key].next_up() == up);
            assert!(index.data[key].next_down() == down);
            assert!(index.data[key].next_left() == left);
            assert!(index.data[key].next_right() == right);
        };

        // Check middle column
        test_cell((0, 1), 0, 2, 0, 2);
        test_cell((1, 1), 0, 2, 0, 2);
        test_cell((2, 1), 0, 2, 0, 2);

        // Check middle row
        test_cell((1, 0), 0, 2, 0, 2);
        test_cell((1, 1), 0, 2, 0, 2);
        test_cell((1, 2), 0, 2, 0, 2);
    }
}
