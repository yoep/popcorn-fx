package com.github.yoep.provider.anime.media.models;

import lombok.Data;

import javax.xml.bind.annotation.XmlAccessType;
import javax.xml.bind.annotation.XmlAccessorType;
import javax.xml.bind.annotation.XmlElement;
import java.util.List;

@Data
@XmlAccessorType(XmlAccessType.FIELD)
public class Channel {
    private String title;
    @XmlElement(name = "item")
    private List<Item> items;
}
