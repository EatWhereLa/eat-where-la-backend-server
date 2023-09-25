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
    timestamp timestamp,

    primary key (user_id, place_id),
    constraint user_favourite_places_fk foreign key (place_id) references places (place_id)
);