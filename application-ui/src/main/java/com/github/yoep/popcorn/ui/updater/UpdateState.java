package com.github.yoep.popcorn.ui.updater;

public enum UpdateState {
    CHECKING_FOR_NEW_VERSION,
    UPDATE_AVAILABLE,
    NO_UPDATE_AVAILABLE,
    DOWNLOADING,
    DOWNLOAD_FINISHED,
    INSTALLING,
    ERROR
}
