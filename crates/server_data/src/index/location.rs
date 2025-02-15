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

use std::{fmt::Debug, num::NonZeroU16, sync::Arc};

use model_server_data::{CellData, CellDataProvider, LocationIndexKey};
use nalgebra::{DMatrix, Dyn, VecStorage};
use tracing::error;

use super::area::LocationIndexArea;

// Finland's area is 338 462 square kilometer, so this is most likely
// good enough value as the iterator does not go all squares one by one.
const INDEX_ITERATOR_COUNT_LIMIT: u32 = 350_000;

/// Origin (0,0) = (y, x) is at top left corner.
pub struct LocationIndex {
    data: DMatrix<CellData>,
}

impl LocationIndex {
    pub fn new(width: NonZeroU16, height: NonZeroU16) -> Self {
        let size = (width.get() as usize) * (height.get() as usize);
        let mut data = Vec::with_capacity(size);
        data.resize_with(size, || CellData::new(width.get(), height.get()));
        let storage = VecStorage::new(Dyn(height.get() as usize), Dyn(width.get() as usize), data);
        Self {
            data: DMatrix::from_data(storage),
        }
    }

    pub fn data(&self) -> &DMatrix<CellData> {
        &self.data
    }
}

impl Debug for LocationIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LocationIndex")
    }
}

pub trait ReadIndex {
    type C: CellDataProvider;

    fn get_cell_data(&self, x: usize, y: usize) -> Option<&Self::C>;

    /// Index width. Greater than zero.
    fn width(&self) -> usize;

    /// Index height. Greater than zero.
    fn height(&self) -> usize;

    fn last_row_index(&self) -> usize {
        self.height() - 1
    }

    fn last_column_index(&self) -> usize {
        self.width() - 1
    }
}

impl <T: AsRef<LocationIndex>> ReadIndex for T {
    type C = CellData;
    fn get_cell_data(&self, x: usize, y: usize) -> Option<&Self::C> {
        self.as_ref().data().get((y, x))
    }

    /// Index width. Greater than zero.
    fn width(&self) -> usize {
        self.as_ref().data().ncols()
    }

    /// Index height. Greater than zero.
    fn height(&self) -> usize {
        self.as_ref().data().nrows()
    }
}

impl AsRef<LocationIndex> for LocationIndex {
    fn as_ref(&self) -> &LocationIndex {
        self
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

#[derive(Debug, Clone, Default)]
struct IndexLimit {
    x: isize,
    y: isize,
}

impl IndexLimit {
    fn new(key: LocationIndexKey) -> Self {
        Self {
            x: key.x as isize,
            y: key.y as isize,
        }
    }
}

/// Iterator for location index
///
/// Start from one cell and enlarge area clockwise.
/// Each iteration starts from one cell down of top right corner.
/// Iteration ends to top right corner.
#[derive(Debug, Clone)]
pub struct LocationIndexIteratorState {
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
    index_limit_top_left: IndexLimit,
    index_limit_bottom_right: IndexLimit,
}

impl LocationIndexIteratorState {
    pub fn completed() -> Self {
        Self {
            y: 0,
            x: 0,
            init_position_y: 0,
            init_position_x: 0,
            iteration_count: 0,
            iter_init_position_x: 0,
            iter_init_position_y: 0,
            direction: Direction::Down,
            completed: true,
            visited_max_corners: VisitedMaxCorners::default(),
            index_limit_top_left: IndexLimit::default(),
            index_limit_bottom_right: IndexLimit::default(),
        }
    }

    pub fn new(
        area: &LocationIndexArea,
        random_start_position: bool,
        index: &impl ReadIndex,
    ) -> Self {
        let start_position = area.index_iterator_start_location(random_start_position);
        let x = (start_position.x as isize).min(index.width() as isize - 1);
        let y = (start_position.y as isize).min(index.height() as isize - 1);

        Self {
            x,
            y,
            init_position_x: x,
            init_position_y: y,
            iter_init_position_x: x,
            iter_init_position_y: y,
            iteration_count: 0,
            direction: Direction::Down,
            completed: false,
            visited_max_corners: VisitedMaxCorners::default(),
            index_limit_top_left: IndexLimit::new(area.top_left),
            index_limit_bottom_right: IndexLimit::new(area.bottom_right),
        }
    }

    pub fn into_iterator<T: ReadIndex>(self, reader: T) -> LocationIndexIterator<T> {
        LocationIndexIterator::new(self, reader)
    }

    /// Get next cell where are profiles.
    ///
    /// Returns key for HashMap. Key is (y, x)
    fn next_raw(&mut self, index: &impl ReadIndex) -> Option<(u16, u16)> {
        if self.completed {
            return None;
        }

        let mut count_iterations = 0;

        loop {
            let data_position = if self.current_cell_has_profiles(index) {
                Some((self.y as u16, self.x as u16))
            } else {
                None
            };

            match self.move_next_position(index) {
                Ok(()) => (),
                Err(()) => {
                    self.completed = true;
                    return data_position;
                }
            }

            if data_position.is_some() {
                return data_position;
            }

            if count_iterations >= INDEX_ITERATOR_COUNT_LIMIT {
                error!(
                    "Location index iterator max count {} reached. This is a bug.",
                    count_iterations,
                );
                self.completed = true;
                return None;
            } else {
                count_iterations += 1;
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

    fn current_cell_has_profiles(&self, index: &impl ReadIndex) -> bool {
        // Make area outside limits appear empty
        if self.x < self.index_limit_top_left.x  ||
            self.x > self.index_limit_bottom_right.x ||
            self.y < self.index_limit_top_left.y ||
            self.y > self.index_limit_bottom_right.y {
            return false;
        }

        self.current_cell(index)
            .map(|cell| cell.profiles())
            .unwrap_or(false)
    }

    fn current_cell<'a, A: ReadIndex>(&self, index: &'a A) -> Option<&'a A::C> {
        let x = self.x.try_into().ok()?;
        let y = self.y.try_into().ok()?;
        index.get_cell_data(x, y)
    }

    /// Move position according to cell next index information.
    ///
    /// Returns error if there is no next new position.
    fn move_next_position(&mut self, index: &impl ReadIndex) -> Result<(), ()> {
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
                if self.y >= index.height() as isize {
                    // Bottom: outside matrix
                    self.y = index.last_row_index() as isize;
                } else if self.y <= 0 {
                    // Top: top line or outside matrix
                    self.y = self.current_top_max_index();
                } else {
                    // Normal: inside matrix area and not the first row.
                    self.y = self
                        .current_cell(index)
                        .map_or(0, |c| c.next_up() as isize)
                        .max(self.current_top_max_index())
                }
            }
            Direction::Down => {
                if self.y >= index.last_row_index() as isize {
                    // Bottom: outside matrix or bottom row
                    self.y = self.current_bottom_max_index();
                } else if self.y < 0 {
                    // Top: top line or outside matrix
                    self.y = 0;
                } else {
                    // Normal: inside matrix area and not the last row.
                    self.y = self
                        .current_cell(index)
                        .map_or(index.last_row_index() as isize, |c| {
                            c.next_down() as isize
                        })
                        .min(self.current_bottom_max_index())
                }
            }
            Direction::Left => {
                if self.x > index.last_column_index() as isize {
                    // Right: outside matrix
                    self.x = index.last_column_index() as isize;
                } else if self.x <= 0 {
                    // Left: left column or outside matrix
                    self.x = self.current_left_max_index();
                } else {
                    // Normal: inside matrix area and not the left column.
                    self.x = self
                        .current_cell(index)
                        .map_or(0, |c| c.next_left() as isize)
                        .max(self.current_left_max_index())
                }
            }
            Direction::Right => {
                if self.x >= index.last_column_index() as isize {
                    // Right: outside matrix or last column
                    self.x = self.current_right_max_index();
                } else if self.x < 0 {
                    // Left: outside matrix
                    self.x = 0;
                } else {
                    // Normal: inside matrix area and not the right column.
                    self.x = self
                        .current_cell(index)
                        .map_or(index.last_column_index() as isize, |c| {
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

    /// Top right corner starts the round
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
        if self.y <= self.index_limit_top_left.y && self.x <= self.index_limit_top_left.x {
            self.visited_max_corners.top_left = true;
        }
        if self.y <= self.index_limit_top_left.y && self.x >= self.index_limit_bottom_right.x {
            self.visited_max_corners.top_right = true;
        }
        if self.y >= self.index_limit_bottom_right.y && self.x <= self.index_limit_top_left.x {
            self.visited_max_corners.bottom_left = true;
        }
        if self.y >= self.index_limit_bottom_right.y && self.x >= self.index_limit_bottom_right.x {
            self.visited_max_corners.bottom_right = true;
        }
    }
}

impl <T: ReadIndex> From<LocationIndexIterator<T>> for LocationIndexIteratorState {
    fn from(value: LocationIndexIterator<T>) -> Self {
        value.state
    }
}

pub struct LocationIndexIterator<T: ReadIndex> {
    state: LocationIndexIteratorState,
    area: T,
}

impl <T: ReadIndex> LocationIndexIterator<T> {
    fn new(
        state: LocationIndexIteratorState,
        area: T,
    ) -> Self {
        Self {
            state,
            area,
        }
    }

    pub fn next_raw(&mut self) -> Option<(u16, u16)> {
        self.state.next_raw(&self.area)
    }
}

impl <T: ReadIndex> Iterator for LocationIndexIterator<T> {
    type Item = LocationIndexKey;

    /// Get next cell where are profiles.
    ///
    /// If None then there is not any more cells with profiles.
    fn next(&mut self) -> Option<LocationIndexKey> {
        self.state.next_raw(&self.area).map(|(y, x)| LocationIndexKey { y, x })
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

    fn max_area(x: u16, y: u16, index: &LocationIndex) -> LocationIndexArea {
        LocationIndexArea::max_area(
            LocationIndexKey { y, x },
            index.width() as u16,
            index.height() as u16
        )
    }

    fn init_with_index(x: u16, y: u16) -> LocationIndexIterator<LocationIndex> {
        let index = index();
        let area = max_area(x, y, &index);
        LocationIndexIterator::new(LocationIndexIteratorState::new(&area, false, &index), index)
    }

    fn init_with_mirror_index(x: u16, y: u16) -> LocationIndexIterator<LocationIndex> {
        let index = mirror_index();
        let area = max_area(x, y, &index);
        LocationIndexIterator::new(LocationIndexIteratorState::new(&area, false, &index), index)
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
        let mut iter = init_with_index(0, 0);

        let n = iter.next_raw();
        assert!(n == Some((0, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 4)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((9, 4)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((9, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n.is_none(), "was: {n:?}");
    }

    #[test]
    fn iterator_top_right_works() {
        let mut iter = init_with_index(4, 0);

        let n = iter.next_raw();
        assert!(n == Some((0, 4)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((9, 4)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((9, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n.is_none(), "was: {n:?}");
    }

    #[test]
    fn iterator_bottom_right_works() {
        let mut iter = init_with_index(4, 9);

        let n = iter.next_raw();
        assert!(n == Some((9, 4)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((9, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 4)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n.is_none(), "was: {n:?}");
    }

    #[test]
    fn iterator_bottom_left_works() {
        let mut iter = init_with_index(0, 9);

        let n = iter.next_raw();
        assert!(n == Some((9, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((9, 4)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 4)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n.is_none(), "was: {n:?}");
    }

    #[test]
    fn mirror_iterator_top_left_works() {
        let mut iter = init_with_mirror_index(0, 0);

        let n = iter.next_raw();
        assert!(n == Some((0, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((4, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 9)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((4, 9)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n.is_none(), "was: {n:?}");
    }

    #[test]
    fn mirror_iterator_top_right_works() {
        let mut iter = init_with_mirror_index(9, 0);

        let n = iter.next_raw();
        assert!(n == Some((0, 9)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((4, 9)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((4, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n.is_none(), "was: {n:?}");
    }

    #[test]
    fn mirror_iterator_bottom_right_works() {
        let mut iter = init_with_mirror_index(9, 4);

        let n = iter.next_raw();
        assert!(n == Some((4, 9)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 9)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((4, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n.is_none(), "was: {n:?}");
    }

    #[test]
    fn mirror_iterator_bottom_left_works() {
        let mut iter = init_with_mirror_index(0, 4);

        let n = iter.next_raw();
        assert!(n == Some((4, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 0)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((0, 9)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n == Some((4, 9)), "was: {n:?}");
        let n = iter.next_raw();
        assert!(n.is_none(), "was: {n:?}");
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
