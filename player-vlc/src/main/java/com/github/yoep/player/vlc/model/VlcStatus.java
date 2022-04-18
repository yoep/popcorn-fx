package com.github.yoep.player.vlc.model;

import lombok.AccessLevel;
import lombok.AllArgsConstructor;
import lombok.Getter;
import lombok.NoArgsConstructor;

import javax.xml.bind.annotation.XmlElement;
import javax.xml.bind.annotation.XmlRootElement;

@Getter
@NoArgsConstructor(access = AccessLevel.PRIVATE)
@AllArgsConstructor
@XmlRootElement(namespace = "", name = "root")
public class VlcStatus {
    @XmlElement
    private Long time;
    @XmlElement
    private Long length;
    /**
     * Volume indication of the player between 0-256 (muted-max).
     */
    @XmlElement
    private Integer volume;
    @XmlElement
    private VlcState state;
}
