use oss_derive::oss_gen_rc;

fn main() {
    assert!(true);
}

#[derive(PartialEq, Debug)]
pub struct ClientArc;

#[derive(PartialEq, Debug)]
#[cfg(feature = "blocking")]
pub struct ClientRc;

#[derive(PartialEq, Debug)]
pub enum Client {
    Arc(ClientArc),
    #[cfg(feature = "blocking")]
    Rc(ClientRc),
}

pub struct Demo<T> {
    pub key: T,
    pub inner: Client,
}

#[oss_gen_rc]
impl Demo<ClientArc> {
    pub fn set_client(&mut self, client: ClientArc) {
        self.inner = Client::Arc(client);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_set_client() {
        use crate::{Client, ClientArc, Demo};

        let mut demo = Demo {
            key: ClientArc,
            inner: Client::Arc(ClientArc),
        };

        demo.set_client(ClientArc);

        assert_eq!(demo.inner, Client::Arc(ClientArc));
    }

    #[cfg(feature = "blocking")]
    #[test]
    fn test_set_client_blocking() {
        use crate::{Client, ClientRc, Demo};

        let mut demo = Demo {
            key: ClientRc,
            inner: Client::Rc(ClientRc),
        };

        demo.set_client(ClientRc);

        assert_eq!(demo.inner, Client::Rc(ClientRc));
    }
}
