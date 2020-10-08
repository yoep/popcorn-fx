package com.github.yoep.popcorn.ui.media.providers.models;

import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.NoArgsConstructor;

import java.io.Serializable;

@Data
@NoArgsConstructor
@AllArgsConstructor
public class Rating implements Serializable {
    private int percentage;
    private int watching;
    private int votes;
    private int loved;
    private int hated;
}
