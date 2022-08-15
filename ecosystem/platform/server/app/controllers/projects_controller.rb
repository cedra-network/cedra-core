# frozen_string_literal: true

# Copyright (c) Aptos
# SPDX-License-Identifier: Apache-2.0

class ProjectsController < ApplicationController
  before_action :authenticate_user!
  before_action :ensure_projects_enabled!
  before_action :ensure_confirmed!
  before_action :set_project, only: %i[show edit update destroy]
  respond_to :html

  # GET /projects
  def index
    @projects = Project.all
  end

  # GET /projects/1
  def show; end

  # GET /projects/new
  def new
    @project = Project.new
    @project.project_categories.new
    @project.project_members.new
    @project.project_screenshots.new
  end

  # GET /projects/1/edit
  def edit; end

  # POST /projects
  def create
    @project = Project.new(project_params)

    return unless check_recaptcha

    if @project.save
      redirect_to project_url(@project), notice: 'Project was successfully created.'
    else
      render :new, status: :unprocessable_entity
    end
  end

  # PATCH/PUT /projects/1
  def update
    return unless check_recaptcha

    if @project.update(project_params)
      redirect_to project_url(@project), notice: 'Project was successfully updated.'
    else
      render :edit, status: :unprocessable_entity
    end
  end

  # DELETE /projects/1
  def destroy
    @project.destroy

    redirect_to projects_url, notice: 'Project was successfully destroyed.'
  end

  private

  # Use callbacks to share common setup or constraints between actions.
  def set_project
    @project = Project.find(params[:id])
  end

  # Only allow a list of trusted parameters through.
  def project_params
    params.require(:project).permit(:title, :short_description, :full_description, :website_url, :github_url,
                                    :discord_url, :twitter_url, :telegram_url, :linkedin_url, :thumbnail_url,
                                    :youtube_url, :public,
                                    project_categories_attributes: %i[id category_id],
                                    project_members_attributes: %i[id user_id role public],
                                    project_screenshots_attributes: %i[id url])
  end

  def check_recaptcha
    recaptcha_v3_success = verify_recaptcha(action: 'projects/update', minimum_score: 0.5,
                                            secret_key: ENV.fetch('RECAPTCHA_V3_SECRET_KEY', nil), model: @project)
    recaptcha_v2_success = verify_recaptcha(model: @project) unless recaptcha_v3_success
    unless recaptcha_v3_success || recaptcha_v2_success
      @show_recaptcha_v2 = true
      respond_with(@project, status: :unprocessable_entity)
      return false
    end
    true
  end

  def ensure_projects_enabled!
    redirect_to root_path unless Flipper.enabled?(:projects)
  end
end
