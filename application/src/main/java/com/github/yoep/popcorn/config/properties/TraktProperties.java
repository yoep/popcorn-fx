package com.github.yoep.popcorn.config.properties;

import lombok.Data;
import org.springframework.security.oauth2.client.token.grant.code.AuthorizationCodeResourceDetails;

import javax.validation.constraints.NotNull;
import java.net.URI;

@Data
public class TraktProperties {
    /**
     * The base url of the trakt.tv API.
     */
    @NotNull
    private URI url;
    /**
     * The client details of the trakt api.
     */
    @NotNull
    private AuthorizationCodeResourceDetails client;
}
