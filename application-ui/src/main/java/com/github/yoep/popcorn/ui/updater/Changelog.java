package com.github.yoep.popcorn.ui.updater;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;

@Data
@Builder
@AllArgsConstructor
public class Changelog {
    private String[] features;
    private String[] bugfixes;
}
