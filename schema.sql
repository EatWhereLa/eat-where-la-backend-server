create table places
(
    place_id        varchar primary key,
    name            varchar,
    photo_height    int,
    photo_width     int,
    photo_reference varchar,
    rating          double precision,
    vicinity        varchar,
    lat             double precision,
    lng             double precision
);

create table user_favourite_places
(
    user_id   varchar,
    place_id  varchar,
    timestamp int,

    primary key (user_id, place_id),
    constraint user_favourite_places_fk foreign key (place_id) references places (place_id)
);

create table user_reviews
(
    user_id  varchar,
    place_id varchar,
    rating   double precision,
    description varchar,
    timestamp timestamp
);

create table user_reservations
(
    user_id               varchar,
    place_id              varchar,
    reservation_timestamp int,
    reservation_pax       int
);

create table voting_history
(
    voted_places   json[],
    vote_timestamp int,
    voters         text[]
);