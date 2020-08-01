package com.github.yoep.popcorn.ui.config.properties;

import lombok.Data;

import javax.validation.constraints.NotNull;
import java.net.URI;
import java.util.List;

@Data
public class ProviderProperties {
    /**
     * The base url of the API that should be used by the provider.
     */
    private List<URI> uris;

    /**
     * The supported genres by the Popcorn API.
     * https://popcornofficial.docs.apiary.io/#reference/genres/page?console=1
     */
    @NotNull
    private List<String> genres;

    /**
     * The supported "sort by" by the Popcorn API.
     */
    @NotNull
    private List<String> sortBy;
}
