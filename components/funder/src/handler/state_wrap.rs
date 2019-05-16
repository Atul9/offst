use std::fmt::Debug;

use common::canonical_serialize::CanonicalSerialize;

use crypto::crypto_rand::CryptoRandom;
use crypto::identity::PublicKey;
use crypto::uid::Uid;

use proto::app_server::messages::RelayAddress;
use proto::funder::messages::FunderOutgoingControl;
use proto::report::messages::{FunderReportMutation, FunderReportMutations};

use identity::IdentityClient;

use crate::state::{FunderMutation, FunderState};

use crate::ephemeral::{Ephemeral, EphemeralMutation};
use crate::friend::ChannelStatus;
use crate::report::{ephemeral_mutation_to_report_mutations, funder_mutation_to_report_mutations};
use crate::types::{ChannelerConfig, FunderIncoming, FunderIncomingComm, FunderOutgoingComm};

pub struct MutableFunderState<B: Clone> {
    initial_state: FunderState<B>,
    state: FunderState<B>,
    mutations: Vec<FunderMutation<B>>,
}

impl<B> MutableFunderState<B>
where
    B: Clone + CanonicalSerialize + PartialEq + Eq + Debug,
{
    pub fn new(state: FunderState<B>) -> Self {
        MutableFunderState {
            initial_state: state.clone(),
            state,
            mutations: Vec::new(),
        }
    }

    pub fn mutate(&mut self, mutation: FunderMutation<B>) {
        self.state.mutate(&mutation);
        self.mutations.push(mutation);
    }

    pub fn state(&self) -> &FunderState<B> {
        &self.state
    }

    pub fn done(self) -> (FunderState<B>, Vec<FunderMutation<B>>, FunderState<B>) {
        (self.initial_state, self.mutations, self.state)
    }
}

pub struct MutableEphemeral {
    ephemeral: Ephemeral,
    mutations: Vec<EphemeralMutation>,
}

impl MutableEphemeral {
    pub fn new(ephemeral: Ephemeral) -> Self {
        MutableEphemeral {
            ephemeral,
            mutations: Vec::new(),
        }
    }
    pub fn mutate(&mut self, mutation: EphemeralMutation) {
        self.ephemeral.mutate(&mutation);
        self.mutations.push(mutation);
    }

    pub fn ephemeral(&self) -> &Ephemeral {
        &self.ephemeral
    }

    pub fn done(self) -> (Vec<EphemeralMutation>, Ephemeral) {
        (self.mutations, self.ephemeral)
    }
}
