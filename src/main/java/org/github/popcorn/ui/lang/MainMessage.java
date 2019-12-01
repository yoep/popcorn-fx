package org.github.popcorn.ui.lang;

import lombok.Getter;

@Getter
public enum MainMessage implements Message {
    ;

    private String key;

    MainMessage(String key) {
        this.key = key;
    }
}
