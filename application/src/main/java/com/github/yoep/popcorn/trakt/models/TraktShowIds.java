package com.github.yoep.popcorn.trakt.models;

import lombok.Data;
import lombok.EqualsAndHashCode;

@EqualsAndHashCode(callSuper = true)
@Data
public class TraktShowIds extends AbstractTraktIds {
    private int tvdb;
}
