package com.github.yoep.popcorn.backend;

import javafx.scene.input.KeyCode;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class BackendConstants {
    public static final String POPCORN_HOME_DIRECTORY = ".popcorn-time";
    public static final String POPCORN_HOME_PROPERTY = ".popcorn-time";

    public static final KeyCode KEEP_ALIVE_SIGNAL = KeyCode.CONTROL;
}
