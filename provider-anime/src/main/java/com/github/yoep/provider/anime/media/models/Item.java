package com.github.yoep.provider.anime.media.models;

import lombok.Data;

import javax.xml.bind.annotation.XmlAccessType;
import javax.xml.bind.annotation.XmlAccessorType;
import javax.xml.bind.annotation.XmlElement;

@Data
@XmlAccessorType(XmlAccessType.FIELD)
public class Item {
    private static final String NYAA_NAMESPACE = "https://nyaa.si/xmlns/nyaa";

    private String title;
    private String link;
    private String guid;
    private String pubDate;
    @XmlElement(namespace = NYAA_NAMESPACE)
    private int seeders;
    @XmlElement(namespace = NYAA_NAMESPACE)
    private int leechers;
    @XmlElement(namespace = NYAA_NAMESPACE)
    private int downloads;
    @XmlElement(namespace = NYAA_NAMESPACE)
    private String infoHash;
    @XmlElement(namespace = NYAA_NAMESPACE)
    private String categoryId;
    @XmlElement(namespace = NYAA_NAMESPACE)
    private String size;
    @XmlElement(namespace = NYAA_NAMESPACE)
    private String comments;
    @XmlElement(namespace = NYAA_NAMESPACE)
    private String trusted;
    @XmlElement(namespace = NYAA_NAMESPACE)
    private String remake;
}
