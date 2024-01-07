use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use apple_music::{AppleMusic, PlayerState, Track};
use dotenv::dotenv;
use rust_discord_activity;
use rust_discord_activity::{DiscordClient, Activity, Payload, EventName, EventData, Asset, Timestamp, Button};
use urlencoding::encode;
use log::debug;

fn main() {
    dotenv().ok();
    env_logger::init();

    let mut discord = DiscordClient::new("1189712673915535452");
    let mut current_track: Option<Track> = None;

    loop {
        let large_img: Option<String>;
        let small_img: Option<String>;
        let application;
        let mut track;
        let mut activity = Activity::new();

        debug!("Getting current track & application data");


        match AppleMusic::get_application_data() {
            Ok(data) => application = data,
            Err(_) => {
                debug!("Could not get Apple Music application data ! Is the app Running ?");
                debug!("Retrying in 30s ...");
                thread::sleep(Duration::from_secs(30));
                continue;
            }
        }

        match application.player_state {
            Some(PlayerState::Stopped) => {
                debug!("No music playing ! Retrying in 30s ...");

                let asset = Asset::new(None, None, Some("https://i.imgur.com/oIOOWnj.png".into()), None);
                activity.set_state(None)
                    .set_details(Some("Not Playling".into()))
                    .set_assets(Some(asset));
                let payload = Payload::new(EventName::Activity, EventData::Activity(activity));
                discord.send_payload(payload).expect("Failed to push Payload to Discord!");

                thread::sleep(Duration::from_secs(30));
                continue;
            }
            _ => {}
        }

        match AppleMusic::get_current_track() {
            Ok(data) => track = data,
            Err(_) => {
                debug!("Could not get current track ! Retrying in 30s ...");
                thread::sleep(Duration::from_secs(30));
                continue;
            }
        }

        if let Some(mut current_track) = current_track {
            if current_track.name != track.name {
                debug!("Track is different");
                large_img = get_artwork_url(&mut track);
                set_buttons(&mut track, &mut activity);
            } else {
                debug!("Track is same");
                large_img = get_artwork_url(&mut current_track);
                set_buttons(&mut current_track, &mut activity);
            }
        } else {
            debug!("First track");
            large_img = get_artwork_url(&mut track);
            set_buttons(&mut track, &mut activity);
        }

        debug!("Matching player_state");
        match application.player_state {
            Some(PlayerState::Playing) => {
                let current_position = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as f64 - application.player_position.unwrap();
                let timestamp = Timestamp::new(Some(current_position as u128), None);
                activity.set_timestamps(Some(timestamp));

                small_img = Some("https://i.imgur.com/QVuAPgP.png".into());
            }
            Some(PlayerState::Paused) => { small_img = Some("https://i.imgur.com/e4wGZxB.png".into()); }
            _ => small_img = Some("https://i.imgur.com/oIOOWnj.png".into())
        }

        let asset = Asset::new(large_img, None, small_img, None);

        activity.set_state(Some(format!("by {}", track.artist)))
            .set_details(Some(track.name.clone()))
            .set_assets(Some(asset));

        let payload = Payload::new(EventName::Activity, EventData::Activity(activity));

        debug!("Pushing payload");
        debug!("{:?}", payload);
        discord.send_payload(payload).expect("Failed to push Payload to Discord!");

        current_track = Some(track);
        debug!("Waiting 2 sec..");
        thread::sleep(Duration::from_secs(5));
    }
}

fn set_buttons(mut track: &mut Track, activity: &mut Activity) {
    let encoded_params = format!("{}+{}+{}", track.name, track.artist, track.album);
    let spotify_url = format!("{}{}", "https://open.spotify.com/search/", encode(encoded_params.as_str()));
    let btn_spotify = Button::new("Search on Spotify".into(), spotify_url);
    let mut btn_vec = vec!(btn_spotify);

    if let Some(url) = get_track_url(&mut track) {
        let btn_applemusic = Button::new("Open album on Apple Music".into(), url);
        btn_vec.insert(0, btn_applemusic);
    }

    activity.set_buttons(Some(btn_vec));
}

fn get_artwork_url(track: &mut Track) -> Option<String> {
    match track.artwork_url().clone() {
        Some(url) => Some(url),
        None => None
    }
}

fn get_track_url(track: &mut Track) -> Option<String> {
    match track.track_url().clone() {
        Some(url) => Some(url),
        None => None
    }
}