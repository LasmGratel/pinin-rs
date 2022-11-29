use smallvec::SmallVec;
use unicode_segmentation::UnicodeSegmentation;

pub trait UnicodeUtils<'a> {
    fn first_grapheme(&'a self) -> &'a str;
    fn remove_first_grapheme(&'a self) -> &'a str;

    fn last_grapheme(&'a self) -> &'a str;
    fn remove_last_grapheme(&'a self) -> &'a str;

    fn substring(&'a self, start: usize, len: usize) -> &'a str;
}

pub struct SegmentedStr<'a> {
    pub raw: &'a str,
    pub graphemes: SmallVec<[(usize, &'a str); 7]>,
}

impl<'a> From<&'a str> for SegmentedStr<'a> {
    fn from(value: &'a str) -> Self {
        SegmentedStr {
            raw: value,
            graphemes: value.grapheme_indices(true).collect(),
        }
    }
}

impl<'a> UnicodeUtils<'a> for SegmentedStr<'a> {
    fn first_grapheme(&'a self) -> &'a str {
        self.graphemes.first().unwrap().1
    }

    fn remove_first_grapheme(&'a self) -> &'a str {
        &self.raw[self.graphemes.first().unwrap().1.len()..]
    }

    fn last_grapheme(&'a self) -> &'a str {
        self.graphemes.last().unwrap().1
    }

    fn remove_last_grapheme(&'a self) -> &'a str {
        &self.raw[..self.graphemes.last().unwrap().0]
    }

    fn substring(&'a self, start: usize, len: usize) -> &'a str {
        let end = self.graphemes[start + len];
        &self.raw[self.graphemes[start].0..end.0 + end.1.len()]
    }
}

impl<'a> UnicodeUtils<'a> for str {
    fn first_grapheme(&'a self) -> &'a str {
        self.graphemes(true).next().unwrap()
    }

    fn remove_first_grapheme(&'a self) -> &'a str {
        &self[self.graphemes(true).next().unwrap().len()..]
    }

    fn last_grapheme(&'a self) -> &'a str {
        self.graphemes(true).last().unwrap()
    }

    fn remove_last_grapheme(&'a self) -> &'a str {
        &self[..self.grapheme_indices(true).last().unwrap().0]
    }

    fn substring(&'a self, start: usize, len: usize) -> &'a str {
        assert!(len > 0, "Length must > 0");

        let mut begin = 0;

        let mut end = 0;
        let mut temp = "";

        for (i, (index, s)) in self
            .grapheme_indices(true)
            .skip(start)
            .take(len)
            .enumerate()
        {
            if i == 0 {
                begin = index;
            }

            end = index;
            temp = s;
        }

        if begin == end {
            return temp;
        }

        &self[begin..end + temp.len()]
    }
}
