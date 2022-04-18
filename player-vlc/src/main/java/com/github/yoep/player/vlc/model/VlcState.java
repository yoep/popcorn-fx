package com.github.yoep.player.vlc.model;

import lombok.Getter;

import javax.xml.bind.annotation.XmlEnum;
import javax.xml.bind.annotation.XmlEnumValue;
import java.util.Arrays;

@Getter
@XmlEnum
public enum VlcState {
    @XmlEnumValue("paused") PAUSED("paused"),
    @XmlEnumValue("playing") PLAYING("playing"),
    @XmlEnumValue("stopped") STOPPED("stopped");

    private final String state;

    VlcState(String state) {
        this.state = state;
    }

    public static VlcState fromValue(String stateValue) {
        return Arrays.stream(values())
                .filter(e -> e.getState().equals(stateValue))
                .findFirst()
                .orElseThrow(() -> new EnumConstantNotPresentException(VlcState.class, stateValue));
    }
}
