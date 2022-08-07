package com.github.yoep.popcorn.ui;

import com.github.yoep.popcorn.Application;
import com.github.yoep.popcorn.PopcornFx;
import org.springframework.stereotype.Component;

import javax.annotation.PreDestroy;

@Component
public class PopcornFxManager {
    private final PopcornFx instance;

    public PopcornFxManager() {
        this.instance = Application.INSTANCE.new_popcorn_fx();
    }

    @PreDestroy
    void onDestroy() {
        Application.INSTANCE.delete_popcorn_fx(instance);
    }
}
