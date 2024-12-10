// originally from: https://github.com/component/rope/blob/master/index.js

use std::fmt;

pub struct ComponentRope {
    value: Option<String>,
    left: Option<Box<ComponentRope>>,
    right: Option<Box<ComponentRope>>,
    length: usize,
}

impl ComponentRope {
    const SPLIT_LENGTH: usize = 1000;
    const JOIN_LENGTH: usize = 500;
    const REBALANCE_RATIO: f64 = 1.2;

    pub fn new(str: String) -> Self {
        let mut rope = Self {
            value: Some(str),
            left: None,
            right: None,
            length: 0,
        };

        if let Some(ref value) = rope.value {
            rope.length = value.len();
        }

        rope.adjust();
        rope
    }

    fn adjust(&mut self) {
        if let Some(ref value) = self.value {
            if self.length > Self::SPLIT_LENGTH {
                let divide = self.length / 2;
                let (left_str, right_str) = value.split_at(divide);

                self.left = Some(Box::new(Self::new(left_str.to_string())));
                self.right = Some(Box::new(Self::new(right_str.to_string())));
                self.value = None;
            }
        } else if self.length < Self::JOIN_LENGTH {
            if let (Some(ref left), Some(ref right)) = (&self.left, &self.right) {
                self.value = Some(format!("{}{}", left.to_string(), right.to_string()));
                self.left = None;
                self.right = None;
            }
        }
    }

    /// Removes text from the rope between the start and end positions.
    /// The character at start gets removed, but the character at end is not removed.
    ///
    /// # Arguments
    ///
    /// * `start` - Initial position (inclusive)
    /// * `end` - Final position (not-inclusive)
    ///
    /// # Panics
    ///
    /// Panics if start or end are out of bounds, or if start > end
    pub fn remove(&mut self, start: usize, end: usize) {
        // Validate bounds
        if start > self.length {
            panic!("Start is not within rope bounds");
        }
        if end > self.length {
            panic!("End is not within rope bounds");
        }
        if start > end {
            panic!("Start is greater than end");
        }

        match &mut self.value {
            Some(value) => {
                // Direct string manipulation for leaf nodes
                let new_value = format!("{}{}", &value[..start], &value[end..]);
                *value = new_value;
                self.length = value.len();
            }
            None => {
                // Handle removal across child nodes
                let left = self
                    .left
                    .as_mut()
                    .expect("Non-leaf node must have left child");
                let right = self
                    .right
                    .as_mut()
                    .expect("Non-leaf node must have right child");

                let left_length = left.length;
                let left_start = start.min(left_length);
                let left_end = end.min(left_length);

                let right_start = (start.saturating_sub(left_length)).min(right.length);
                let right_end = (end.saturating_sub(left_length)).min(right.length);

                // Remove from left child if necessary
                if left_start < left_length {
                    left.remove(left_start, left_end);
                }

                // Remove from right child if necessary
                if right_end > 0 {
                    right.remove(right_start, right_end);
                }

                self.length = left.length + right.length;
            }
        }

        self.adjust();
    }

    /// Inserts text into the rope at the specified position.
    ///
    /// # Arguments
    ///
    /// * `position` - Where to insert the text
    /// * `value` - Text to be inserted into the rope
    ///
    /// # Panics
    ///
    /// Panics if position is out of bounds
    pub fn insert(&mut self, position: usize, value: &str) {
        if position > self.length {
            panic!("Position is not within rope bounds");
        }

        match &mut self.value {
            Some(existing_value) => {
                // Direct string manipulation for leaf nodes
                let new_value = format!(
                    "{}{}{}",
                    &existing_value[..position],
                    value,
                    &existing_value[position..]
                );
                *existing_value = new_value;
                self.length = existing_value.len();
            }
            None => {
                // Handle insertion across child nodes
                let left = self
                    .left
                    .as_mut()
                    .expect("Non-leaf node must have left child");
                let right = self
                    .right
                    .as_mut()
                    .expect("Non-leaf node must have right child");

                let left_length = left.length;
                if position < left_length {
                    left.insert(position, value);
                } else {
                    right.insert(position - left_length, value);
                }

                self.length = left.length + right.length;
            }
        }

        self.adjust();
    }

    /// Rebuilds the entire rope structure, producing a balanced tree.
    pub fn rebuild(&mut self) {
        if self.value.is_none() {
            let left = self
                .left
                .take()
                .expect("Non-leaf node must have left child");
            let right = self
                .right
                .take()
                .expect("Non-leaf node must have right child");

            // Combine the strings from left and right children
            self.value = Some(format!("{}{}", left.to_string(), right.to_string()));
            // Left and right are automatically dropped here since we took ownership

            self.adjust();
        }
    }

    /// Finds unbalanced nodes in the tree and rebuilds them.
    pub fn rebalance(&mut self) {
        if self.value.is_none() {
            let left = self
                .left
                .as_ref()
                .expect("Non-leaf node must have left child");
            let right = self
                .right
                .as_ref()
                .expect("Non-leaf node must have right child");

            let left_len = left.length as f64;
            let right_len = right.length as f64;

            if left_len / right_len > Self::REBALANCE_RATIO
                || right_len / left_len > Self::REBALANCE_RATIO
            {
                self.rebuild();
            } else {
                // Need to get mutable references after the ratio check
                let left = self.left.as_mut().unwrap();
                let right = self.right.as_mut().unwrap();
                left.rebalance();
                right.rebalance();
            }
        }
    }

    /// Returns text from the rope between the start and end positions.
    /// The character at start gets returned, but the character at end is not returned.
    ///
    /// # Arguments
    ///
    /// * `start` - Initial position (inclusive)
    /// * `end` - Final position (not-inclusive), defaults to rope length if None
    pub fn substring(&self, start: isize, end: Option<isize>) -> String {
        // Convert and bound start position
        let start = if start < 0 {
            0
        } else {
            start.min(self.length as isize) as usize
        };

        // Convert and bound end position
        let end = match end {
            None => self.length,
            Some(e) => {
                if e < 0 {
                    0
                } else {
                    e.min(self.length as isize) as usize
                }
            }
        };

        match &self.value {
            Some(value) => value[start..end].to_string(),
            None => {
                let left = self
                    .left
                    .as_ref()
                    .expect("Non-leaf node must have left child");
                let right = self
                    .right
                    .as_ref()
                    .expect("Non-leaf node must have right child");

                let left_length = left.length;
                let left_start = start.min(left_length);
                let left_end = end.min(left_length);
                let right_start = (start.saturating_sub(left_length)).min(right.length);
                let right_end = (end.saturating_sub(left_length)).min(right.length);

                match (left_start != left_end, right_start != right_end) {
                    (true, true) => format!(
                        "{}{}",
                        left.substring(left_start as isize, Some(left_end as isize)),
                        right.substring(right_start as isize, Some(right_end as isize))
                    ),
                    (true, false) => left.substring(left_start as isize, Some(left_end as isize)),
                    (false, true) => {
                        right.substring(right_start as isize, Some(right_end as isize))
                    }
                    (false, false) => String::new(),
                }
            }
        }
    }

    /// Returns a string of length characters from the rope, starting at the start position.
    ///
    /// # Arguments
    ///
    /// * `start` - Initial position (inclusive)
    /// * `length` - Size of the string to return, defaults to remaining length if None
    pub fn substr(&self, mut start: isize, length: Option<isize>) -> String {
        if start < 0 {
            start = (self.length as isize + start).max(0);
        }

        let end = match length {
            None => self.length as isize,
            Some(len) => {
                if len < 0 {
                    0
                } else {
                    start + len
                }
            }
        };

        self.substring(start, Some(end))
    }

    /// Returns the character at the given position.
    ///
    /// # Arguments
    ///
    /// * `position` - The position of the character to return
    pub fn char_at(&self, position: isize) -> String {
        self.substring(position, Some(position + 1))
    }

    /// Returns the Unicode code point of the character at the given position.
    ///
    /// # Arguments
    ///
    /// * `position` - The position of the character to get the code point for
    ///
    /// # Panics
    ///
    /// Panics if the position is out of bounds or if the substring is empty
    pub fn char_code_at(&self, position: isize) -> u32 {
        let ch = self.substring(position, Some(position + 1));
        ch.chars().next().expect("Invalid position").into()
    }
}

impl fmt::Display for ComponentRope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Some(value) => write!(f, "{}", value),
            None => {
                let left = self
                    .left
                    .as_ref()
                    .expect("Non-leaf node must have left child");
                let right = self
                    .right
                    .as_ref()
                    .expect("Non-leaf node must have right child");
                write!(f, "{}{}", left, right)
            }
        }
    }
}
