package com.github.yoep.provider.anime.media.models;

import lombok.Data;

import javax.xml.bind.annotation.XmlAccessType;
import javax.xml.bind.annotation.XmlAccessorType;
import javax.xml.bind.annotation.XmlElement;
import javax.xml.bind.annotation.XmlRootElement;

@Data
@XmlRootElement(name = "rss")
@XmlAccessorType(XmlAccessType.FIELD)
public class Nyaa {
    @XmlElement(name = "channel")
    private Channel channel;
}
