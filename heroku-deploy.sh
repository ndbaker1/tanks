#!/bin/bash

APP_NAME=${1:-"rust-tanks"}

echo "Starting deploy for Heroku :: $APP_NAME"

heroku container:login
heroku container:push -a $APP_NAME web
heroku container:release -a $APP_NAME web