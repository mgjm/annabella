use std::{
    borrow::Cow, cell::RefCell, collections::BTreeMap, marker::PhantomData, ops::Range,
    path::PathBuf, ptr::NonNull,
};

#[cfg(feature = "span-locations")]
pub(super) type SpanOffset = u32;

#[cfg(not(feature = "span-locations"))]
pub(super) type SpanOffset = ();

#[cfg(feature = "span-locations")]
const ZERO_INDEX: SpanOffset = 0;

#[cfg(not(feature = "span-locations"))]
const ZERO_INDEX: SpanOffset = ();

type NotSendOrSync = PhantomData<NonNull<()>>;

pub trait Spanned {
    fn span(&self) -> Span;

    fn first_span(&self) -> Span {
        self.span()
    }

    fn recoverable_error(&self, msg: impl Into<Cow<'static, str>>) -> crate::parser::Error {
        crate::parser::Error::recoverable(self.span(), msg)
    }

    fn unrecoverable_error(&self, msg: impl Into<Cow<'static, str>>) -> crate::parser::Error {
        crate::parser::Error::unrecoverable(self.span(), msg)
    }
}

impl<T: Spanned + ?Sized> Spanned for &T {
    fn span(&self) -> Span {
        T::span(*self)
    }
}

impl<T: Spanned> Spanned for &mut T {
    fn span(&self) -> Span {
        T::span(*self)
    }
}

impl<T: Spanned> Spanned for Option<T> {
    fn span(&self) -> Span {
        self.as_ref().map_or(Span::call_site(), T::span)
    }
}

impl<T: Spanned> Spanned for [T] {
    fn span(&self) -> Span {
        let mut span = Span::call_site();
        for item in self {
            span.extend(item.span());
        }
        span
    }

    fn first_span(&self) -> Span {
        self.first().map_or(Span::call_site(), Spanned::span)
    }
}

impl<A: Spanned, B: Spanned> Spanned for (A, B) {
    fn span(&self) -> Span {
        let mut span = self.0.span();
        span.extend(self.1.span());
        span
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub(super) start: SpanOffset,
    pub(super) end: SpanOffset,
    _not_send_or_sync: NotSendOrSync,
}

impl Span {
    pub const fn call_site() -> Self {
        Self::new(ZERO_INDEX, ZERO_INDEX)
    }

    pub(super) const fn new(start: SpanOffset, end: SpanOffset) -> Self {
        Self {
            start,
            end,
            _not_send_or_sync: PhantomData,
        }
    }

    pub fn for_new_file(source: String, filename: Option<PathBuf>) -> Self {
        FILE_INFOS.with_borrow_mut(|files| files.push(source, filename))
    }

    pub fn start(self) -> Self {
        Self::new(self.start, self.start)
    }

    pub fn end(self) -> Self {
        Self::new(self.end, self.end)
    }

    pub fn join(self, other: Self) -> Option<Self> {
        #[cfg(feature = "span-locations")]
        {
            if self.is_call_site() {
                Some(other)
            } else if other.is_call_site() {
                Some(self)
            } else if FILE_INFOS.with_borrow(|files| files.get(self).span_within(other)) {
                Some(Self::new(
                    u32::min(self.start, other.start),
                    u32::max(self.end, other.end),
                ))
            } else {
                None
            }
        }

        #[cfg(not(feature = "span-locations"))]
        {
            let _ = other;
            Some(Self::call_site())
        }
    }

    pub fn filepath(self) -> Option<PathBuf> {
        #[cfg(feature = "span-locations")]
        {
            FILE_INFOS.with_borrow(|files| files.get(self).path.clone())
        }

        #[cfg(not(feature = "span-locations"))]
        {
            None
        }
    }

    #[cfg(feature = "span-locations")]
    pub fn start_line_column(self) -> LineColumn {
        FILE_INFOS.with_borrow(|files| files.get(self).offset_line_column(self.start))
    }

    #[cfg(feature = "span-locations")]
    pub fn end_line_column(self) -> LineColumn {
        FILE_INFOS.with_borrow(|files| files.get(self).offset_line_column(self.end))
    }

    pub fn is_call_site(self) -> bool {
        self.start == ZERO_INDEX && self.end == ZERO_INDEX
    }

    pub fn source(self) -> Option<String> {
        #[cfg(feature = "span-locations")]
        if !self.is_call_site() {
            return Some(FILE_INFOS.with_borrow_mut(|files| files.get_mut(self).source(self)));
        }

        None
    }

    pub fn lines(self) -> Vec<(usize, String, Option<u32>, Option<u32>)> {
        FILE_INFOS.with_borrow_mut(|files| {
            let file = files.get_mut(self);
            let start = file.offset_line_column(self.start);
            let end = file.offset_line_column(self.end);
            (start.line..=end.line)
                .map(|line| {
                    let source = file.source(Span::new(
                        file.span.start + file.lines[line - 1],
                        file.span.start + file.lines.get(line).map_or(file.span.end, |o| o - 2),
                    ));
                    assert!(!source.ends_with('\n'));
                    let start = if line == start.line {
                        Some(start.column)
                    } else {
                        None
                    };
                    let end = if line == end.line {
                        Some(end.column)
                    } else {
                        None
                    };
                    (line, source, start, end)
                })
                .collect()
        })
    }

    pub fn extend(&mut self, span: Span) {
        if let Some(joined) = self.join(span) {
            *self = joined;
        }
    }
}

impl Spanned for Span {
    fn span(&self) -> Span {
        *self
    }
}

#[cfg(feature = "span-locations")]
thread_local! {
    static FILE_INFOS: RefCell<FileInfos> = RefCell::default();
}

#[cfg(feature = "span-locations")]
struct FileInfos {
    files: Vec<FileInfo>,
}

#[cfg(feature = "span-locations")]
impl Default for FileInfos {
    fn default() -> Self {
        Self {
            files: vec![FileInfo::call_site()],
        }
    }
}

#[cfg(feature = "span-locations")]
impl FileInfos {
    fn next_start(&self) -> u32 {
        self.files.last().unwrap().span.end + 1
    }

    fn push(&mut self, source: String, path: Option<PathBuf>) -> Span {
        let (len, lines) = Self::lines_offsets(&source);
        let start = self.next_start();
        let end = start + len;
        let span = Span::new(start, end);
        self.files.push(FileInfo {
            source,
            span,
            lines,
            char_offset_to_byte_index: BTreeMap::new(),
            path,
        });
        span
    }

    fn lines_offsets(s: &str) -> (u32, Vec<u32>) {
        assert!(s.len() <= u32::MAX as usize);

        let mut offset = 0;
        let mut lines = vec![offset];

        for ch in s.chars() {
            offset += 1;
            if ch == '\n' {
                lines.push(offset);
            }
        }

        (offset, lines)
    }

    fn get(&self, span: Span) -> &FileInfo {
        self.files
            .iter()
            .find(|file| file.span_within(span))
            .expect("invalid span range, no file info found")
    }

    fn get_mut(&mut self, span: Span) -> &mut FileInfo {
        self.files
            .iter_mut()
            .find(|file| file.span_within(span))
            .expect("invalid span range, no file info found")
    }
}

#[cfg(feature = "span-locations")]
pub struct FileInfo {
    source: String,
    span: Span,
    lines: Vec<SpanOffset>,
    char_offset_to_byte_index: BTreeMap<u32, usize>,
    path: Option<PathBuf>,
}

#[cfg(feature = "span-locations")]
impl FileInfo {
    fn call_site() -> Self {
        FileInfo {
            source: String::new(),
            span: Span::new(0, 0),
            lines: vec![0],
            char_offset_to_byte_index: BTreeMap::new(),
            path: None,
        }
    }

    fn offset_line_column(&self, offset: SpanOffset) -> LineColumn {
        assert!(self.span_within(Span::new(offset, offset)));
        let offset = offset - self.span.start;
        match self.lines.binary_search(&offset) {
            Ok(index) => LineColumn {
                line: index + 1,
                column: 0,
            },
            Err(index) => LineColumn {
                line: index,
                column: offset - self.lines[index - 1],
            },
        }
    }

    fn span_within(&self, offset: Span) -> bool {
        self.span.start <= offset.start && offset.end <= self.span.end
    }

    fn source(&mut self, span: Span) -> String {
        let byte_range = self.byte_range(span);
        self.source[byte_range].into()
    }

    fn byte_range(&mut self, span: Span) -> Range<usize> {
        let start_char_offset = span.start - self.span.start;

        let (&last_char_offset, &last_byte_index) = self
            .char_offset_to_byte_index
            .range(..=start_char_offset)
            .next_back()
            .unwrap_or((&0, &0));

        let start_byte_index;
        if last_char_offset == start_char_offset {
            start_byte_index = last_byte_index;
        } else {
            start_byte_index = self.char_offset_to_byte_index_with_last(
                last_char_offset,
                last_byte_index,
                start_char_offset,
            );
            self.char_offset_to_byte_index
                .insert(start_char_offset, start_byte_index);
        };

        let end_char_offset = span.end;
        let end_byte_index = self.char_offset_to_byte_index_with_last(
            start_char_offset,
            start_byte_index,
            end_char_offset,
        );
        start_byte_index..end_byte_index
    }

    fn char_offset_to_byte_index_with_last(
        &self,
        last_char_offset: u32,
        last_byte_index: usize,
        char_offset: u32,
    ) -> usize {
        match self.source[last_byte_index..]
            .char_indices()
            .nth((char_offset - last_char_offset) as usize)
        {
            Some((byte_index, _)) => last_byte_index + byte_index,
            None => self.source.len(),
        }
    }
}

#[cfg(feature = "span-locations")]
/// A line-column pair represents the source position of a span offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineColumn {
    /// The 1-indexec line in the source file.
    line: usize,

    /// The 0-indexed column (in characters) in the source file.
    column: u32,
}

pub fn offset_add(offset: SpanOffset, num: impl Fn() -> usize) -> SpanOffset {
    #[cfg(feature = "span-locations")]
    {
        offset + num() as u32
    }

    #[cfg(not(feature = "span-locations"))]
    {
        let _ = (offset, num);
    }
}
