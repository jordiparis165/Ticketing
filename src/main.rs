use std::collections::HashMap;

use serde::{Deserialize, Serialize};

type ArtistId = u64;
type VenueId = u64;
type ConcertId = u64;
type TicketId = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    pub id: ArtistId,
    pub name: String,
    pub artist_type: String,
    pub total_tickets_sold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Venue {
    pub id: VenueId,
    pub name: String,
    pub capacity: u32,
    pub venue_cut_bps: u16,
    pub next_concert_date: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concert {
    pub id: ConcertId,
    pub artist_id: ArtistId,
    pub venue_id: VenueId,
    pub date_ts: u64,
    pub ticket_price: u64,
    pub total_tickets: u32,
    pub tickets_issued: u32,
    pub validated_by_artist: bool,
    pub validated_by_venue: bool,
    pub tickets_sold: u32,
    pub revenue: u64,
    pub cashed_out: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: TicketId,
    pub concert_id: ConcertId,
    pub owner: Option<String>,
    pub used: bool,
    pub price_paid: u64,
    pub minted_by_artist: bool,
    pub redeem_code: Option<String>,
}

#[derive(Default)]
pub struct Ticketing {
    next_artist_id: ArtistId,
    next_venue_id: VenueId,
    next_concert_id: ConcertId,
    next_ticket_id: TicketId,

    artists: HashMap<ArtistId, Artist>,
    venues: HashMap<VenueId, Venue>,
    concerts: HashMap<ConcertId, Concert>,
    tickets: HashMap<TicketId, Ticket>,

    balances_artist: HashMap<ArtistId, u64>,
    balances_venue: HashMap<VenueId, u64>,
}

impl Ticketing {
    pub fn create_artist(&mut self, name: &str, artist_type: &str) -> ArtistId {
        self.next_artist_id += 1;
        let id = self.next_artist_id;
        self.artists.insert(
            id,
            Artist {
                id,
                name: name.to_string(),
                artist_type: artist_type.to_string(),
                total_tickets_sold: 0,
            },
        );
        id
    }

    pub fn update_artist(&mut self, id: ArtistId, name: &str, artist_type: &str) {
        if let Some(artist) = self.artists.get_mut(&id) {
            artist.name = name.to_string();
            artist.artist_type = artist_type.to_string();
        }
    }

    pub fn create_venue(
        &mut self,
        name: &str,
        capacity: u32,
        venue_cut_bps: u16,
        next_concert_date: Option<u64>,
    ) -> VenueId {
        self.next_venue_id += 1;
        let id = self.next_venue_id;
        self.venues.insert(
            id,
            Venue {
                id,
                name: name.to_string(),
                capacity,
                venue_cut_bps,
                next_concert_date,
            },
        );
        id
    }

    pub fn update_venue(
        &mut self,
        id: VenueId,
        name: &str,
        capacity: u32,
        venue_cut_bps: u16,
        next_concert_date: Option<u64>,
    ) {
        if let Some(venue) = self.venues.get_mut(&id) {
            venue.name = name.to_string();
            venue.capacity = capacity;
            venue.venue_cut_bps = venue_cut_bps;
            venue.next_concert_date = next_concert_date;
        }
    }

    pub fn create_concert(
        &mut self,
        artist_id: ArtistId,
        venue_id: VenueId,
        date_ts: u64,
        ticket_price: u64,
        total_tickets: u32,
    ) -> ConcertId {
        self.next_concert_id += 1;
        let id = self.next_concert_id;
        self.concerts.insert(
            id,
            Concert {
                id,
                artist_id,
                venue_id,
                date_ts,
                ticket_price,
                total_tickets,
                tickets_issued: 0,
                validated_by_artist: false,
                validated_by_venue: false,
                tickets_sold: 0,
                revenue: 0,
                cashed_out: false,
            },
        );
        id
    }

    pub fn validate_concert_by_artist(&mut self, concert_id: ConcertId, artist_id: ArtistId) {
        if let Some(concert) = self.concerts.get_mut(&concert_id) {
            if concert.artist_id == artist_id {
                concert.validated_by_artist = true;
            }
        }
    }

    pub fn validate_concert_by_venue(&mut self, concert_id: ConcertId, venue_id: VenueId) {
        if let Some(concert) = self.concerts.get_mut(&concert_id) {
            if concert.venue_id == venue_id {
                concert.validated_by_venue = true;
            }
        }
    }

    pub fn emit_ticket(
        &mut self,
        concert_id: ConcertId,
        artist_id: ArtistId,
        redeem_code: Option<String>,
    ) -> Option<TicketId> {
        let concert = self.concerts.get(&concert_id)?;
        if concert.artist_id != artist_id || !concert.validated_by_artist || !concert.validated_by_venue {
            return None;
        }
        if concert.tickets_issued >= concert.total_tickets {
            return None;
        }

        self.next_ticket_id += 1;
        let ticket_id = self.next_ticket_id;
        self.tickets.insert(
            ticket_id,
            Ticket {
                id: ticket_id,
                concert_id,
                owner: Some(format!("artist:{artist_id}")),
                used: false,
                price_paid: 0,
                minted_by_artist: true,
                redeem_code,
            },
        );
        if let Some(concert) = self.concerts.get_mut(&concert_id) {
            concert.tickets_issued += 1;
        }
        Some(ticket_id)
    }

    pub fn buy_ticket(&mut self, concert_id: ConcertId, buyer: &str, amount_paid: u64) -> Option<TicketId> {
        let concert = self.concerts.get_mut(&concert_id)?;
        if !concert.validated_by_artist || !concert.validated_by_venue {
            return None;
        }
        if concert.tickets_issued >= concert.total_tickets {
            return None;
        }
        concert.tickets_sold = concert.tickets_sold.saturating_add(1);
        concert.tickets_issued = concert.tickets_issued.saturating_add(1);
        concert.revenue = concert.revenue.saturating_add(amount_paid);

        self.next_ticket_id += 1;
        let ticket_id = self.next_ticket_id;
        self.tickets.insert(
            ticket_id,
            Ticket {
                id: ticket_id,
                concert_id,
                owner: Some(buyer.to_string()),
                used: false,
                price_paid: amount_paid,
                minted_by_artist: false,
                redeem_code: None,
            },
        );
        if let Some(artist) = self.artists.get_mut(&concert.artist_id) {
            artist.total_tickets_sold += 1;
        }
        Some(ticket_id)
    }

    pub fn transfer_ticket(&mut self, ticket_id: TicketId, from: &str, to: &str) -> bool {
        if let Some(ticket) = self.tickets.get_mut(&ticket_id) {
            if ticket.owner.as_deref() == Some(from) && !ticket.used {
                ticket.owner = Some(to.to_string());
                return true;
            }
        }
        false
    }

    pub fn use_ticket(&mut self, ticket_id: TicketId, owner: &str, now_ts: u64) -> bool {
        let concert_id = match self.tickets.get(&ticket_id) {
            Some(ticket) if ticket.owner.as_deref() == Some(owner) && !ticket.used => ticket.concert_id,
            _ => return false,
        };
        let concert = match self.concerts.get(&concert_id) {
            Some(c) => c,
            None => return false,
        };
        if !(concert.validated_by_artist && concert.validated_by_venue) {
            return false;
        }
        let window_start = concert.date_ts.saturating_sub(86_400);
        if now_ts >= window_start && now_ts <= concert.date_ts {
            if let Some(ticket) = self.tickets.get_mut(&ticket_id) {
                ticket.used = true;
                return true;
            }
        }
        false
    }

    pub fn cash_out(&mut self, concert_id: ConcertId, now_ts: u64) -> bool {
        let (artist_id, venue_id, revenue, venue_cut_bps, cashed_out, date_ts) = match self.concerts.get(&concert_id) {
            Some(c) => (c.artist_id, c.venue_id, c.revenue, self.venues.get(&c.venue_id).map(|v| v.venue_cut_bps).unwrap_or(0), c.cashed_out, c.date_ts),
            None => return false,
        };
        if cashed_out || now_ts < date_ts {
            return false;
        }
        let venue_cut = revenue * venue_cut_bps as u64 / 10_000;
        let artist_cut = revenue.saturating_sub(venue_cut);
        *self.balances_artist.entry(artist_id).or_default() += artist_cut;
        *self.balances_venue.entry(venue_id).or_default() += venue_cut;
        if let Some(c) = self.concerts.get_mut(&concert_id) {
            c.cashed_out = true;
        }
        true
    }

    pub fn trade_ticket(
        &mut self,
        ticket_id: TicketId,
        seller: &str,
        buyer: &str,
        price: u64,
    ) -> bool {
        let ticket = match self.tickets.get_mut(&ticket_id) {
            Some(t) => t,
            None => return false,
        };
        if ticket.owner.as_deref() != Some(seller) || ticket.used {
            return false;
        }
        if price > ticket.price_paid {
            return false;
        }
        ticket.owner = Some(buyer.to_string());
        ticket.price_paid = price;
        true
    }

    pub fn distribute_ticket(
        &mut self,
        concert_id: ConcertId,
        artist_id: ArtistId,
        redeem_code: &str,
    ) -> Option<TicketId> {
        let concert = self.concerts.get_mut(&concert_id)?;
        if concert.artist_id != artist_id || !concert.validated_by_artist || !concert.validated_by_venue {
            return None;
        }
        if concert.tickets_issued >= concert.total_tickets {
            return None;
        }
        concert.tickets_issued = concert.tickets_issued.saturating_add(1);

        self.next_ticket_id += 1;
        let ticket_id = self.next_ticket_id;
        self.tickets.insert(
            ticket_id,
            Ticket {
                id: ticket_id,
                concert_id,
                owner: None,
                used: false,
                price_paid: 0,
                minted_by_artist: true,
                redeem_code: Some(redeem_code.to_string()),
            },
        );
        Some(ticket_id)
    }

    pub fn redeem_ticket(&mut self, code: &str, user: &str) -> Option<TicketId> {
        let target = self
            .tickets
            .iter_mut()
            .find(|(_, t)| t.owner.is_none() && t.redeem_code.as_deref() == Some(code));
        if let Some((id, ticket)) = target {
            ticket.owner = Some(user.to_string());
            return Some(*id);
        }
        None
    }

    pub fn ticket_owner(&self, ticket_id: TicketId) -> Option<String> {
        self.tickets.get(&ticket_id).and_then(|t| t.owner.clone())
    }

    pub fn balance_artist(&self, artist_id: ArtistId) -> u64 {
        *self.balances_artist.get(&artist_id).unwrap_or(&0)
    }

    pub fn balance_venue(&self, venue_id: VenueId) -> u64 {
        *self.balances_venue.get(&venue_id).unwrap_or(&0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_validated_concert(ticket_limit: u32) -> (Ticketing, ConcertId, ArtistId, VenueId) {
        let mut t = Ticketing::default();
        let artist = t.create_artist("Artist", "band");
        let venue = t.create_venue("Venue", 1_000, 1_000, None);
        let concert = t.create_concert(artist, venue, 1_000_000, 100, ticket_limit);
        t.validate_concert_by_artist(concert, artist);
        t.validate_concert_by_venue(concert, venue);
        (t, concert, artist, venue)
    }

    #[test]
    fn enforces_supply_across_sales_and_distributions() {
        let (mut t, concert, artist, _) = setup_validated_concert(2);
        assert!(t.buy_ticket(concert, "alice", 100).is_some());
        assert!(t.distribute_ticket(concert, artist, "FREE1").is_some());
        assert!(t.emit_ticket(concert, artist, None).is_none(), "supply cap must block extra tickets");
        assert!(t.buy_ticket(concert, "bob", 100).is_none(), "supply cap must block further sales");
    }

    #[test]
    fn redeem_distributed_ticket() {
        let (mut t, concert, artist, _) = setup_validated_concert(1);
        let ticket_id = t.distribute_ticket(concert, artist, "CODE123").expect("ticket minted");
        assert_eq!(t.redeem_ticket("CODE123", "carol"), Some(ticket_id));
        assert_eq!(t.ticket_owner(ticket_id).as_deref(), Some("carol"));
        assert_eq!(t.redeem_ticket("CODE123", "dave"), None, "code unusable after redeem");
    }

    #[test]
    fn use_ticket_respects_window_and_validation() {
        let (mut t, concert, _, _) = setup_validated_concert(2);
        let early_ticket = t.buy_ticket(concert, "eve", 100).unwrap();
        assert!(!t.use_ticket(early_ticket, "eve", 1_000_000 - 86_401));
        assert!(t.use_ticket(early_ticket, "eve", 1_000_000 - 10));
        assert!(!t.use_ticket(early_ticket, "eve", 1_000_000 - 5), "cannot reuse");

        // After event, usage should fail
        let late_ticket = t.buy_ticket(concert, "frank", 100).unwrap();
        assert!(!t.use_ticket(late_ticket, "frank", 1_000_000 + 1));
    }

    #[test]
    fn trade_ticket_never_above_purchase_price() {
        let (mut t, concert, artist, _) = setup_validated_concert(3);
        let paid_ticket = t.buy_ticket(concert, "gina", 100).unwrap();
        assert!(!t.trade_ticket(paid_ticket, "gina", "helen", 120), "cannot sell above paid price");
        assert!(t.trade_ticket(paid_ticket, "gina", "helen", 80));

        // Free ticket cannot be resold for profit
        let free_ticket = t.emit_ticket(concert, artist, None).unwrap();
        assert!(!t.trade_ticket(free_ticket, &format!("artist:{artist}"), "ian", 10));
        assert!(t.trade_ticket(free_ticket, &format!("artist:{artist}"), "ian", 0));
    }
}

fn main() {
    println!("Ticketing module ready. Add your own tests or wire it to a CLI.");
}
