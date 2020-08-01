package com.github.yoep.popcorn.ui.trakt;

import com.fasterxml.jackson.datatype.jsr310.deser.LocalDateTimeDeserializer;

import java.time.format.DateTimeFormatter;

public class TraktDateTimeDeserializer extends LocalDateTimeDeserializer {
    public TraktDateTimeDeserializer() {
        super(DateTimeFormatter.ofPattern("yyyy-MM-dd'T'HH:mm:ss.SSSX"));
    }
}
