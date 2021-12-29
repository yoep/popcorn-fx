package com.github.yoep.popcorn.backend.settings.models;

import lombok.*;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.Optional;

@Slf4j
@EqualsAndHashCode(callSuper = false)
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class TraktSettings extends AbstractSettings {
    public static final String ACCESS_TOKEN_PROPERTY = "accessToken";

    /**
     * The Trakt.tv access token that has been retrieved.
     */
    private OAuth2AccessTokenWrapper accessToken;

    //region Getters & Setters

    /**
     * Get the access token for trakt if known.
     *
     * @return Returns the trakt.tv access token, or else {@link Optional#empty()}.
     */
    public Optional<OAuth2AccessTokenWrapper> getAccessToken() {
        return Optional.ofNullable(accessToken);
    }

    /**
     * Set the nex access token for trakt.tv.
     *
     * @param accessToken The new access token.
     */
    public void setAccessToken(OAuth2AccessTokenWrapper accessToken) {
        if (Objects.equals(this.accessToken, accessToken))
            return;

        log.trace("Access token has been updated");
        var oldValue = this.accessToken;
        this.accessToken = accessToken;
        changes.firePropertyChange(ACCESS_TOKEN_PROPERTY, oldValue, accessToken);
    }

    //endregion
}
