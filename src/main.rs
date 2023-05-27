use futures::stream::TryStreamExt;
use netlink_packet_route::address::Nla as AddressNla;
use netlink_packet_route::link::nlas::Nla;
use rtnetlink::{new_connection, Error, Handle};

struct Link {
    name: String,
    index: u32,
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    env_logger::init();
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    let link = "eth0".to_string();
    // Dump all the links and print their index and name
    println!("*** All the Interface links ***");
    if let Err(e) = all_links(handle.clone()).await {
        eprintln!("{e}");
    } else {
        for link in all_links(handle.clone()).await.unwrap() {
            println!("Index: {} Name: {}", link.index, link.name);
        }
    }

    println!("dumping address for link \"{link}\"");

    if let Err(e) = get_address_of_link(handle.clone(), link.clone()).await {
        eprintln!("{e}");
    } else {
        for address_message in get_address_of_link(handle.clone(), link.clone())
            .await
            .unwrap()
        {
            println!("{address_message:?}");
        }
    }

    Ok(())
}

async fn all_links(handle: Handle) -> Result<Vec<Link>, Error> {
    let mut links = handle.link().get().execute();
    let mut links_vec: Vec<Link> = Vec::new();

    while let Some(msg) = links.try_next().await? {
        for nla in msg.nlas {
            if let Nla::IfName(name) = nla {
                links_vec.push(Link {
                    index: msg.header.index,
                    name: name,
                });
            }
        }
    }

    Ok(links_vec)
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct AddressMessage {
    pub header: AddressHeader,
    pub nlas: Vec<AddressNla>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct AddressHeader {
    pub family: u8,
    pub prefix_len: u8,
    pub flags: u8,
    pub scope: u8,
    pub index: u32,
}

async fn get_address_of_link(handle: Handle, link: String) -> Result<Vec<AddressMessage>, Error> {
    let mut links = handle.link().get().match_name(link.clone()).execute();

    let mut address_of_link: Vec<AddressMessage> = Vec::new();
    if let Some(link) = links.try_next().await? {
        let mut addresses = handle
            .address()
            .get()
            .set_link_index_filter(link.header.index)
            .execute();

        while let Some(msg) = addresses.try_next().await? {
            address_of_link.push({
                AddressMessage {
                    header: AddressHeader {
                        family: msg.header.family,
                        prefix_len: msg.header.prefix_len,
                        flags: msg.header.flags,
                        scope: msg.header.scope,
                        index: msg.header.index,
                    },
                    nlas: msg.nlas,
                }
            });
        }
        Ok(address_of_link)
    } else {
        eprintln!("link {link} not found");
        Ok(Vec::new())
    }
}
