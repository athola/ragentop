//! Event tracking and ring-buffer storage.
//!
//! Pure data structures for agent events and a generic circular buffer.
//! No I/O - the buffer is an in-memory data structure operated on by pure functions.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::SystemTime;

use crate::SessionId;

/// A discrete event from an agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Event {
    /// What kind of event this is.
    pub kind: EventKind,
    /// Human-readable name/description.
    pub name: String,
    /// Arbitrary key-value attributes.
    pub attributes: HashMap<String, String>,
    /// When the event occurred.
    pub timestamp: SystemTime,
    /// Which session produced this event, if known.
    pub session_id: Option<SessionId>,
    /// Whether this event represents a successful outcome.
    pub success: Option<bool>,
}

/// Classification of agent events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EventKind {
    /// User submitted a prompt.
    UserPrompt,
    /// Tool returned a result.
    ToolResult,
    /// API request was made.
    ApiRequest,
    /// API returned an error.
    ApiError,
    /// Agent decided which tool to use.
    ToolDecision,
}

impl std::fmt::Display for EventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserPrompt => write!(f, "user_prompt"),
            Self::ToolResult => write!(f, "tool_result"),
            Self::ApiRequest => write!(f, "api_request"),
            Self::ApiError => write!(f, "api_error"),
            Self::ToolDecision => write!(f, "tool_decision"),
        }
    }
}

impl Event {
    /// Creates a new `Event`.
    #[must_use]
    pub fn new(kind: EventKind, name: impl Into<String>, timestamp: SystemTime) -> Self {
        Self {
            kind,
            name: name.into(),
            attributes: HashMap::new(),
            timestamp,
            session_id: None,
            success: None,
        }
    }

    /// Sets the session ID.
    #[must_use]
    pub fn with_session_id(mut self, session_id: Option<SessionId>) -> Self {
        self.session_id = session_id;
        self
    }

    /// Sets the success flag.
    #[must_use]
    pub const fn with_success(mut self, success: Option<bool>) -> Self {
        self.success = success;
        self
    }

    /// Adds a key-value attribute.
    #[must_use]
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

/// A fixed-capacity circular buffer.
///
/// When the buffer is full, pushing a new element evicts the oldest.
/// Iteration order is oldest-to-newest.
#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    buf: VecDeque<T>,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    /// Creates a new ring buffer with the given capacity.
    ///
    /// # Panics
    /// This function does not panic. A capacity of 0 creates a buffer
    /// that silently drops all pushed items.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Creates a ring buffer with the default capacity of 1000.
    #[must_use]
    pub fn with_default_capacity() -> Self {
        Self::new(1000)
    }

    /// Push an item into the buffer, evicting the oldest if full.
    pub fn push(&mut self, item: T) {
        if self.capacity == 0 {
            return;
        }
        if self.buf.len() >= self.capacity {
            self.buf.pop_front();
        }
        self.buf.push_back(item);
    }

    /// Returns an iterator over items from oldest to newest.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.buf.iter()
    }

    /// Returns the number of items currently in the buffer.
    #[must_use]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Returns the maximum capacity.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns true if the buffer contains no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Removes all items from the buffer.
    pub fn clear(&mut self) {
        self.buf.clear();
    }
}

impl<T> Default for RingBuffer<T> {
    fn default() -> Self {
        Self::with_default_capacity()
    }
}

/// Criteria for filtering events.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EventFilter {
    /// Only include events from this session.
    pub session_id: Option<SessionId>,
    /// Only include events of these kinds. Empty means all kinds.
    pub event_kinds: HashSet<EventKind>,
    /// Only include successful events.
    pub success_only: bool,
    /// Only include failed events.
    pub failure_only: bool,
}

/// Filter a slice of events according to the given criteria.
///
/// Returns references to events that match all filter conditions.
/// An empty `event_kinds` set means no kind filtering is applied.
///
/// # Examples
/// ```
/// use ragentop_core::event::{filter_events, Event, EventFilter, EventKind};
/// use std::time::SystemTime;
///
/// let events = vec![
///     Event::new(EventKind::UserPrompt, "ask", SystemTime::now())
///         .with_success(Some(true)),
/// ];
/// let filter = EventFilter::default();
/// let matched = filter_events(&events, &filter);
/// assert_eq!(matched.len(), 1);
/// ```
#[must_use]
pub fn filter_events<'a>(events: &'a [Event], filter: &EventFilter) -> Vec<&'a Event> {
    events
        .iter()
        .filter(|event| {
            // Session filter
            if let Some(ref sid) = filter.session_id {
                if event.session_id.as_ref() != Some(sid) {
                    return false;
                }
            }

            // Kind filter (empty means all)
            if !filter.event_kinds.is_empty() && !filter.event_kinds.contains(&event.kind) {
                return false;
            }

            // Success/failure filters
            if filter.success_only && event.success != Some(true) {
                return false;
            }
            if filter.failure_only && (event.success == Some(true) || event.success.is_none()) {
                return false;
            }

            true
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- RingBuffer --

    #[test]
    fn ringbuffer_new_is_empty() {
        let buf: RingBuffer<i32> = RingBuffer::new(10);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.capacity(), 10);
    }

    #[test]
    fn ringbuffer_default_capacity_is_1000() {
        let buf: RingBuffer<i32> = RingBuffer::default();
        assert_eq!(buf.capacity(), 1000);
    }

    #[test]
    fn ringbuffer_push_and_len() {
        let mut buf = RingBuffer::new(5);
        buf.push(1);
        buf.push(2);
        buf.push(3);
        assert_eq!(buf.len(), 3);
        assert!(!buf.is_empty());
    }

    #[test]
    fn ringbuffer_overflow_evicts_oldest() {
        let mut buf = RingBuffer::new(3);
        buf.push(1);
        buf.push(2);
        buf.push(3);
        buf.push(4); // Evicts 1
        assert_eq!(buf.len(), 3);
        let items: Vec<&i32> = buf.iter().collect();
        assert_eq!(items, vec![&2, &3, &4]);
    }

    #[test]
    fn ringbuffer_multiple_overflows() {
        let mut buf = RingBuffer::new(2);
        for i in 0..10 {
            buf.push(i);
        }
        assert_eq!(buf.len(), 2);
        let items: Vec<&i32> = buf.iter().collect();
        assert_eq!(items, vec![&8, &9]);
    }

    #[test]
    fn ringbuffer_clear() {
        let mut buf = RingBuffer::new(5);
        buf.push(1);
        buf.push(2);
        buf.clear();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.capacity(), 5);
    }

    #[test]
    fn ringbuffer_iter_order_oldest_to_newest() {
        let mut buf = RingBuffer::new(5);
        buf.push(10);
        buf.push(20);
        buf.push(30);
        let items: Vec<&i32> = buf.iter().collect();
        assert_eq!(items, vec![&10, &20, &30]);
    }

    #[test]
    fn ringbuffer_zero_capacity_drops_items() {
        let mut buf = RingBuffer::new(0);
        buf.push(1);
        buf.push(2);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn ringbuffer_capacity_one() {
        let mut buf = RingBuffer::new(1);
        buf.push(1);
        assert_eq!(buf.len(), 1);
        buf.push(2);
        assert_eq!(buf.len(), 1);
        let items: Vec<&i32> = buf.iter().collect();
        assert_eq!(items, vec![&2]);
    }

    // -- EventKind --

    #[test]
    fn event_kind_display() {
        assert_eq!(EventKind::UserPrompt.to_string(), "user_prompt");
        assert_eq!(EventKind::ToolResult.to_string(), "tool_result");
        assert_eq!(EventKind::ApiRequest.to_string(), "api_request");
        assert_eq!(EventKind::ApiError.to_string(), "api_error");
        assert_eq!(EventKind::ToolDecision.to_string(), "tool_decision");
    }

    #[test]
    fn event_kind_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let kinds = [
            EventKind::UserPrompt,
            EventKind::ToolResult,
            EventKind::ApiRequest,
            EventKind::ApiError,
            EventKind::ToolDecision,
        ];
        for kind in kinds {
            let json = serde_json::to_string(&kind)?;
            let parsed: EventKind = serde_json::from_str(&json)?;
            assert_eq!(parsed, kind);
        }
        Ok(())
    }

    // -- Event --

    fn make_event(kind: EventKind, success: Option<bool>, session_id: Option<SessionId>) -> Event {
        Event {
            kind,
            name: "test-event".to_string(),
            attributes: HashMap::new(),
            timestamp: SystemTime::UNIX_EPOCH,
            session_id,
            success,
        }
    }

    #[test]
    fn event_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let event = make_event(EventKind::UserPrompt, Some(true), None);
        let json = serde_json::to_string(&event)?;
        let parsed: Event = serde_json::from_str(&json)?;
        assert_eq!(parsed.kind, EventKind::UserPrompt);
        assert_eq!(parsed.name, "test-event");
        assert_eq!(parsed.success, Some(true));
        Ok(())
    }

    // -- filter_events --

    #[test]
    fn filter_events_no_filter_returns_all() {
        let events = vec![
            make_event(EventKind::UserPrompt, Some(true), None),
            make_event(EventKind::ApiError, Some(false), None),
        ];
        let filter = EventFilter::default();
        let result = filter_events(&events, &filter);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_events_by_kind() {
        let events = vec![
            make_event(EventKind::UserPrompt, Some(true), None),
            make_event(EventKind::ApiError, Some(false), None),
            make_event(EventKind::UserPrompt, Some(true), None),
        ];
        let filter = EventFilter {
            event_kinds: HashSet::from([EventKind::UserPrompt]),
            ..EventFilter::default()
        };
        let result = filter_events(&events, &filter);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|e| e.kind == EventKind::UserPrompt));
    }

    #[test]
    fn filter_events_by_session_id() -> Result<(), Box<dyn std::error::Error>> {
        let sid = SessionId::new("sess-1")?;
        let events = vec![
            make_event(EventKind::UserPrompt, None, Some(sid.clone())),
            make_event(EventKind::UserPrompt, None, None),
            make_event(EventKind::UserPrompt, None, Some(SessionId::new("sess-2")?)),
        ];
        let filter = EventFilter {
            session_id: Some(sid),
            ..EventFilter::default()
        };
        let result = filter_events(&events, &filter);
        assert_eq!(result.len(), 1);
        Ok(())
    }

    #[test]
    fn filter_events_success_only() {
        let events = vec![
            make_event(EventKind::ToolResult, Some(true), None),
            make_event(EventKind::ToolResult, Some(false), None),
            make_event(EventKind::ToolResult, None, None),
        ];
        let filter = EventFilter {
            success_only: true,
            ..EventFilter::default()
        };
        let result = filter_events(&events, &filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].success, Some(true));
    }

    #[test]
    fn filter_events_failure_only() {
        let events = vec![
            make_event(EventKind::ToolResult, Some(true), None),
            make_event(EventKind::ToolResult, Some(false), None),
            make_event(EventKind::ToolResult, None, None),
        ];
        let filter = EventFilter {
            failure_only: true,
            ..EventFilter::default()
        };
        let result = filter_events(&events, &filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].success, Some(false));
    }

    #[test]
    fn filter_events_empty_input() {
        let events: Vec<Event> = vec![];
        let filter = EventFilter::default();
        let result = filter_events(&events, &filter);
        assert!(result.is_empty());
    }

    #[test]
    fn filter_events_combined_filters() -> Result<(), Box<dyn std::error::Error>> {
        let sid = SessionId::new("sess-1")?;
        let events = vec![
            make_event(EventKind::UserPrompt, Some(true), Some(sid.clone())),
            make_event(EventKind::ApiError, Some(true), Some(sid.clone())),
            make_event(EventKind::UserPrompt, Some(false), Some(sid.clone())),
            make_event(
                EventKind::UserPrompt,
                Some(true),
                Some(SessionId::new("other")?),
            ),
        ];
        let filter = EventFilter {
            session_id: Some(sid),
            event_kinds: HashSet::from([EventKind::UserPrompt]),
            success_only: true,
            ..EventFilter::default()
        };
        let result = filter_events(&events, &filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, EventKind::UserPrompt);
        assert_eq!(result[0].success, Some(true));
        Ok(())
    }
}
