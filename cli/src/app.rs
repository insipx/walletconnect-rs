use argh::FromArgs;
use walletconnect_lib::pairing::PairingUri;
use walletconnect_rpc::types::*;

/// Example pairing CLI for WalletConnect
#[derive(FromArgs)]
pub struct App {
    /// the pairing URI
    #[argh(option, short = 'p')]
    pub pairing_uri: PairingUri<'static>,
}
