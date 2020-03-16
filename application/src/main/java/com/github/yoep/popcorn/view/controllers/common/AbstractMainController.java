package com.github.yoep.popcorn.view.controllers.common;

import com.github.spring.boot.javafx.ui.scale.ScaleAwareImpl;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.view.controllers.MainController;
import com.github.yoep.popcorn.view.services.UrlService;
import javafx.fxml.FXML;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.ApplicationArguments;
import org.springframework.core.task.TaskExecutor;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
public abstract class AbstractMainController extends ScaleAwareImpl implements MainController {
    protected final ActivityManager activityManager;
    protected final ViewLoader viewLoader;
    protected final ApplicationArguments arguments;
    protected final UrlService urlService;
    protected final TaskExecutor taskExecutor;

    protected Pane contentPane;
    protected Pane playerPane;
    protected Pane loaderPane;
    protected Pane notificationPane;

    @FXML
    protected Pane rootPane;

    //region Constructors

    protected AbstractMainController(ActivityManager activityManager,
                                     ViewLoader viewLoader,
                                     ApplicationArguments arguments,
                                     UrlService urlService,
                                     TaskExecutor taskExecutor) {
        Assert.notNull(activityManager, "activityManager cannot be null");
        Assert.notNull(viewLoader, "viewLoader cannot be null");
        Assert.notNull(arguments, "arguments cannot be null");
        Assert.notNull(urlService, "urlService cannot be null");
        Assert.notNull(taskExecutor, "taskExecutor cannot be null");
        this.activityManager = activityManager;
        this.viewLoader = viewLoader;
        this.arguments = arguments;
        this.urlService = urlService;
        this.taskExecutor = taskExecutor;
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeNotificationPane();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializePanesInternal();
        initializePanes();
        initializeListeners();
    }

    private void initializePanesInternal() {
        // load the content & notification pane on the main thread
        // this blocks Spring from completing the startup stage while these panes are being loaded
        contentPane = viewLoader.load("sections/content.section.fxml");
        notificationPane = viewLoader.load("common/sections/notification.section.fxml");

        // load the other panes on a different thread
        taskExecutor.execute(() -> {
            playerPane = viewLoader.load("sections/player.section.fxml");
            loaderPane = viewLoader.load("sections/loader.section.fxml");
        });
    }

    protected abstract void initializePanes();

    protected abstract void initializeListeners();

    //endregion

    //region Functions

    protected boolean processApplicationArguments() {
        var nonOptionArgs = arguments.getNonOptionArgs();

        if (nonOptionArgs.size() > 0) {
            log.debug("Retrieved the following non-option argument: {}", nonOptionArgs);

            // try to process the url that has been passed along the application during startup
            // if the url is processed with success, wait for the activity event to change the section
            // otherwise, we still show the content section
            return urlService.process(nonOptionArgs.get(0));
        }

        return false;
    }

    private void initializeNotificationPane() {
        AnchorPane.setTopAnchor(notificationPane, 55.0);
        AnchorPane.setRightAnchor(notificationPane, 20.0);

        rootPane.getChildren().add(notificationPane);
    }

    //endregion
}
