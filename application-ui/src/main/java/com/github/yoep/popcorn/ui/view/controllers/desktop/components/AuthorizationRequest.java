package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

public interface AuthorizationRequest {
    /**
     * Get the authorization url to redirect the user to.
     *
     * @return Returns the authorization url.
     */
    String getAuthorizationUrl();

    /**
     * Get the expected redirect url.
     *
     * @return Returns the expected redirect url.
     */
    String getRedirectUrl();

    /**
     * Is invoked when the expected redirect url is matched.
     *
     * @param url the url that matched the expected redirect url.
     */
    void onComplete(String url);
}
