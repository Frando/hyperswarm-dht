use crate::kbucket::KeyBytes;
use crate::rpc::message::{Command, Message, Type};
use crate::rpc::query::bootstrap::BootstrapPeersIter;
use crate::rpc::{Node, Peer, PeerId, RequestId};
use fnv::FnvHashMap;
use futures::task::Poll;
use libp2p_kad;
use std::net::SocketAddr;
use wasm_timer::Instant;

mod bootstrap;
mod peers;
pub mod table;

/// A `QueryPool` provides an aggregate state machine for driving `Query`s to completion.
pub struct QueryPool<TInner> {
    queries: FnvHashMap<QueryId, QueryStream<TInner>>,
    next_id: usize,
}

impl<TInner> QueryPool<TInner> {
    /// Returns an iterator over the queries in the pool.
    pub fn iter(&self) -> impl Iterator<Item = &QueryStream<TInner>> {
        self.queries.values()
    }

    /// Gets the current size of the pool, i.e. the number of running queries.
    pub fn size(&self) -> usize {
        self.queries.len()
    }

    fn next_query_id(&mut self) -> QueryId {
        let id = QueryId(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);
        id
    }

    /// Adds a query to the pool.
    pub fn add_command(&mut self) -> QueryId {
        unimplemented!()
    }

    /// Returns a reference to a query with the given ID, if it is in the pool.
    pub fn get(&self, id: &QueryId) -> Option<&QueryStream<TInner>> {
        self.queries.get(id)
    }

    /// Returns a mutable reference to a query with the given ID, if it is in the pool.
    pub fn get_mut(&mut self, id: &QueryId) -> Option<&mut QueryStream<TInner>> {
        self.queries.get_mut(id)
    }

    /// Polls the pool to advance the queries.
    pub fn poll(&mut self, now: Instant) -> QueryPoolState<TInner> {
        if self.queries.is_empty() {
            return QueryPoolState::Idle;
        } else {
            return QueryPoolState::Waiting(None);
        }
        unimplemented!()
    }
}

/// The observable states emitted by [`QueryPool::poll`].
pub enum QueryPoolState<'a, TInner> {
    /// The pool is idle, i.e. there are no queries to process.
    Idle,
    /// At least one query is waiting for results. `Some(request)` indicates
    /// that a new request is now being waited on.
    Waiting(Option<&'a mut QueryStream<TInner>>),
    /// A query has finished.
    Finished(QueryStream<TInner>),
    /// A query has timed out.
    Timeout(QueryStream<TInner>),
}

pub struct QueryStream<TInner> {
    // TODO vecdeque with msgs or PeerIter structs?
    id: QueryId,
    /// The peer iterator that drives the query state.
    peer_iter: QueryPeerIter,
    cmd: Command,
    /// The internal query state.
    state: QueryState,
    stats: QueryStats,
    ty: QueryType,
    /// The opaque inner query state.
    pub inner: TInner,
}

impl<TInner> QueryStream<TInner> {
    // TODO return data
    fn inject_response(&mut self) -> Option<()> {
        unimplemented!()
    }

    pub fn on_bootstrap(&mut self, nodes: &[PeerId]) {
        self.state = QueryState::MovingCloser;
    }

    fn move_closer(&mut self) {
        if self.ty.is_update() {
            self.state = QueryState::Updating;
        } else {
            self.state = QueryState::Finalized;
        }
    }

    // TODO tick call 5000?
    pub fn poll(&mut self) -> Option<QueryEvent> {
        match self.state {
            QueryState::Bootstrapping => {}
            QueryState::MovingCloser => {}
            QueryState::Updating => {}
            QueryState::Finalized => {}
        }

        None
    }
}

/// The peer selection strategies that can be used by queries.
enum QueryPeerIter {
    Bootstrap(BootstrapPeersIter),
    MovingCloser,
    Updating,
}

/// Execution statistics of a query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryStats {
    requests: u32,
    success: u32,
    failure: u32,
    start: Option<Instant>,
    end: Option<Instant>,
}

#[derive(Debug, Clone)]
pub enum QueryType {
    Query,
    Update,
    QueryUpdate,
}

impl QueryType {
    pub fn is_query(&self) -> bool {
        match self {
            QueryType::Query | QueryType::QueryUpdate => true,
            _ => false,
        }
    }

    pub fn is_update(&self) -> bool {
        match self {
            QueryType::Update | QueryType::QueryUpdate => true,
            _ => false,
        }
    }
}

pub enum QueryEvent {
    /// Request including retries failed completely
    Finished,
    Bootstrap {
        target: Vec<u8>,
        num: usize,
    },
    RemoveNode {
        id: Vec<u8>,
    },
    Response {
        ty: Type,
        to: Option<SocketAddr>,
        id: Option<Vec<u8>>,
        peer: Peer,
        value: Option<Vec<u8>>,
        cmd: Command,
    },
}

struct QueryPeer {
    id: Vec<u8>,
    addr: SocketAddr,
    queried: bool,
    distance: u64,
}

#[derive(Debug, Clone)]
enum QueryState {
    Bootstrapping,
    MovingCloser,
    Updating,
    Finalized,
}

impl QueryState {
    fn is_updating(&self) -> bool {
        match self {
            QueryState::Updating => true,
            _ => false,
        }
    }

    fn is_finalized(&self) -> bool {
        match self {
            QueryState::Finalized => true,
            _ => false,
        }
    }

    fn is_moving_closer(&self) -> bool {
        match self {
            QueryState::MovingCloser => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Query {
    /// Whether this a query/update response
    pub ty: Type,
    /// Command def
    pub command: String,
    /// the node who sent the query/update
    // TODO change to `node`?
    pub node: Peer,
    /// the query/update target (32 byte target)
    pub target: Option<Vec<u8>>,
    /// the query/update payload decoded with the inputEncoding
    pub value: Option<Vec<u8>>,
}

/// Unique identifier for an active query.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct QueryId(usize);
