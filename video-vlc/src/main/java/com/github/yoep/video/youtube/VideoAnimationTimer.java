package com.github.yoep.video.youtube;

import javafx.animation.AnimationTimer;

class VideoAnimationTimer extends AnimationTimer {
    private final Runnable handleAction;

    VideoAnimationTimer(Runnable handleAction) {
        this.handleAction = handleAction;
    }

    @Override
    public void handle(long now) {
        handleAction.run();
    }
}
