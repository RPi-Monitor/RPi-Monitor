// This file is part of RPi-Monitor project
//
// Copyright 2013 - Xavier Berger - http://rpi-experiences.blogspot.fr/
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
var animate;
var shellinaboxuri;
var statusautorefresh;
var refreshTimerId;
var clickId;
var current_path = window.location.pathname.split('/').pop();

function GetURLParameter(sParam) {
  var sPageURL = window.location.search.substring(1);
  var sURLVariables = sPageURL.split('&');
  for (var i = 0; i < sURLVariables.length; i++) {
    var sParameterName = sURLVariables[i].split('=');
    if (sParameterName[0] == sParam) {
      return sParameterName[1];
    }
  }
}

function getData(name) {
  if (localStorage.getItem(name + 'Version') == localStorage.getItem('version')) {
    return eval("(" + localStorage.getItem(name) + ')');
  }
  else {
    return $.ajax({
      url: name + '.json',
      dataType: 'json',
      async: false,
      success: function (data) {
        localStorage.setItem(name, JSON.stringify(data))
        localStorage.setItem(name + 'Version', localStorage.getItem('version'))
        return data
      },
      fail: function () {
        $('#message').html("<b>Can not get information (<a href='" + name.json + "'>" + name + ".json</a>) from RPi-Monitor server.</b>");
        $('#message').removeClass('hide');
        return null
      }
    }).responseJSON
  }
}

function ShowFriends() {
  var data = getData('friends')
  if (data.length > 0) {
    $('#friends').empty();
    for (var i = 0; i < data.length; i++) {
      $('#friends').append('<li><a href="' + data[i].link + '">' + eval(data[i].title) + '</a></li>');
    }
    $('#divfriends').removeClass('hide');
  }
}

function AddFooter() {
  $('#footer').html(
    '<div class="navbar-inverse navbar-fixed-bottom text-center">' +
    '<small>' +
    '<a href="http://rpi-experiences.blogspot.fr/">RPi-Experiences</a>' +
    '<font color="silver"> | </font>' +
    '<a href="https://github.com/RPi-Monitor/RPi-Monitor">GitHub</a>' +
    '<font color="silver"> | </font>' +
    '<a href="http://www.raspberrypi.org/">Raspberry Pi Foundation</a>' +
    '</small>' +
    '</div>'
  );
}

function AddDialogs() {
  var dialogs = "";

  // Add Options Dialog
  dialogs +=
    '<div id="Options" class="modal fade">' +
    '<div class="modal-dialog">' +
    '<div class="modal-content">' +
    '<div class="modal-header">' +
    '<button type="button" class="close" data-dismiss="modal"><span aria-hidden="true">&times;</span><span class="sr-only">Close</span></button>' +
    '<h4 id="myModalLabel">Options</h4>' +
    '</div>' +
    '<div class="modal-body">' +
    '<i id="optionsInsertionPoint"></i>' +
    '</div>' +
    '<div class="modal-footer">' +
    '<button class="btn" data-dismiss="modal" aria-hidden="true" id="closeoptions">Close</button>' +
    '</div>' +
    '</div>' +
    '</div>' +
    '</div>';

  // Add License Dialog
  dialogs +=
    '<div id="License" class="modal fade">' +
    '<div class="modal-dialog">' +
    '<div class="modal-content">' +
    '<div class="modal-header">' +
    '<button type="button" class="close" data-dismiss="modal"><span aria-hidden="true">&times;</span><span class="sr-only">Close</span></button>' +
    '<h4 id="myModalLabel">License</h4>' +
    '</div>' +
    '<div class="modal-body">' +
    'This program is free software: you can redistribute it and/or modify ' +
    ' of the GNU General Public License as published ' +
    'by the Free Software Foundation, either version 3 of the License, or ' +
    '(at your option) any later version.<br>' +
    '<br>' +
    'This program is distributed in the hope that it will be useful, but ' +
    'WITHOUT ANY WARRANTY; without even the implied warranty of ' +
    'MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. ' +
    'See the GNU General Public License for more details.<br>' +
    '<br>' +
    'You should have received a copy of the GNU General Public License ' +
    'along with this program. If not, see <a href="http://www.gnu.org/licenses/">http://www.gnu.org/licenses/</a>.' +
    '</p>' +
    '<hr>' +
    '<b>RPi-Monitor</b> is using third party software that have their own licenses. ' +
    'Refer to <a href="#About" data-dismiss="modal" data-toggle="modal">About</a> to view the list of software used by <b>RPi-Monitor</b>. ' +
    '</div>' +
    '<div class="modal-footer">' +
    '<button class="btn" data-dismiss="modal" aria-hidden="true">Close</button>' +
    '</div>' +
    '</div>' +
    '</div>' +
    '</div>';

  // Add About Dialog
  dialogs+=
    '<div id="About" class="modal fade">'+
      '<div class="modal-dialog">' +
        '<div class="modal-content">'+
      '<div class="modal-header">'+
      '<button type="button" class="close" data-dismiss="modal"><span aria-hidden="true">&times;</span><span class="sr-only">Close</span></button>' +
      '<h4 id="myModalLabel">About</h4>'+
      '</div>'+
      '<div class="modal-body">'+
      '<p><b>Version</b>: {DEVELOPMENT} '+
      '<b>by</b> Xavier Berger</p>'+
      'With the contribution of users sharing ideas and competences on Github.'+
      '<br>'+
      '<a href="http://rpi-experiences.blogspot.fr/">Blog</a>'+' - '+
      '<a href="https://github.com/RPi-Monitor/RPi-Monitor">GitHub</a>' + ' - ' +
      '<a href="https://rpi-monitor.github.io/RPi-Monitor/index.html">Documentation</a>'+
      '<hr>'+
      '<p><b>RPi-Monitor</b> is free software developed on top of other open source '+
        'tools: <a href="http://twitter.github.io/bootstrap/">bootstrap</a>, <a href="http://jquery.com/">jquery</a>, <a href="https://code.google.com/p/jsqrencode/">jsqrencode</a>, <a href="http://javascriptrrd.sourceforge.net/">javascriptrrd</a> and <a href="http://www.flotcharts.org/">Flot</a>.<br>'+
      '<p><b>Raspberry Pi</b> and the Raspberry Pi logo are properties of <a href="http://www.raspberrypi.org/">Raspberry Pi Foundation</a>.</p>'+
      '</div>'+
      '<div class="modal-footer">'+
      '<button class="btn" data-dismiss="modal" aria-hidden="true">Close</button>'+
      '</div>'+
      '</div>'+
    '</div>'+
    '</div>';

  $('#dialogs').html(dialogs);
}

function AddTopmenu() {
  page = getData('page')
  data = getData('static')
  try {
    document.title = eval(page.pagetitle);
  }
  catch (err) {
    document.title = page.pagetitle;
  }
  try {
    icon = eval(page.icon);
  }
  catch (err) {
    icon = page.icon;
  }
  try {
    menutitle = eval(page.menutitle);
  }
  catch (err) {
    menutitle = page.menutitle;
  }
  topmenu =
    '<nav class="navbar navbar-inverse navbar-fixed-top" role="navigation">' +
    '<div class="container-fluid">' +
    '<div class="navbar-header">' +
    '<button type="button" class="navbar-toggle collapsed" data-toggle="collapse" data-target="#bs-example-navbar-collapse-1">' +
    '<span class="sr-only">Toggle navigation</span>' +
    '<span class="icon-bar"></span>' +
    '<span class="icon-bar"></span>' +
    '<span class="icon-bar"></span>' +
    '</button>' +
    '<a class="navbar-brand" href="index.html"><img height="20" src="' + icon + '"> &nbsp;' + menutitle + '</a>' +
    '</div>' +
    '<div class="collapse navbar-collapse" id="bs-example-navbar-collapse-1">' +
      '<ul class="nav navbar-nav">' +
        '<li id="statusmenu"><a id="statuslink" href="status.html">Status</a></li>'+
            '<li id="statisticsmenu"><a id="statisticslink" href="statistics.html">Statistics</a></li>'+
            '<li id="addonsmenu" class="hide"><a id="addonslink" href="addons.html">Add-ons</a></li>'+
            '<li id="optionsmenu"><a href="#Options" data-toggle="modal">Options</a></li>'+
            '<li class="dropdown">' +
        '<a href="#" class="dropdown-toggle" data-toggle="dropdown">About <span class="caret"></span></a>' +
        '<ul class="dropdown-menu" role="menu">' +
        '<li class="dropdown-header"> <b>RPi-Monitor</b></li>'+
        '<li><a href="#" title="Scan this qrcode to reach this page from your smartphone or tablet"><canvas id="qrcanv"><a></li>'+
        '<li><a href="#License" data-toggle="modal">License</a></li>'+
        '<li><a href="#About" data-toggle="modal">About</a></li>'+
        '<li class="divider"></li>'+
        '<li class="dropdown-header"> <b>Related links</b></li>'+
        '<li><a href="https://rpi-monitor.github.io/RPi-Monitor/index.html" data-toggle="modal">Documentation</a></li>'+
        '<li><a href="http://rpi-experiences.blogspot.fr/">RPi-Experiences</a></li>'+
        '<li><a href="https://github.com/RPi-Monitor/RPi-Monitor">RPi-Monitor on GitHub</a></li>'+
        '</ul>' +
      '</li>' +
      '</ul>' +
      '<div class="pull-right hide" id="divfriends">'+
        '<ul class="nav navbar-nav">'+
        '<li class="dropdown">'+
          '<a href="#" class="dropdown-toggle" data-toggle="dropdown">Friends <b class="caret"></b></a>'+
          '<ul class="dropdown-menu dropdown-menu-right" id="friends">'+
          '</ul>'+
        '</li>'+
        '</ul>'+
      '</div>'+
    '</div><!-- /.navbar-collapse -->' +
    '</div><!-- /.container-fluid -->' +
    '</nav>'
  $('#topmenu').html(topmenu);
}

function UpdateMenu() {
  // Disable index page
  //var index=true;
  var index = false;

  // Manage active link
  if (current_path == 'status.html') {
    $('#statusmenu').addClass('active');
    index = false;
  }
  else if (current_path == 'statistics.html') {
    $('#statisticsmenu').addClass('active');
    index = false;
  }
  else if (current_path == 'addons.html') {
    $('#addonsmenu').addClass('active');
    index = false;
  }

  // On home page, the menu is not shown
  if (index == true) {
    $('#statusmenu').addClass('hide');
    $('#statisticsmenu').addClass('hide');
    $('#addonsmenu').addClass('hide');
    $('#optionsmenu').addClass('hide');
    return;
  }

  var data = getData('menu');
  if (data.status == undefined) {
    $('#statusmenu').addClass('hide');
  }
  else {
    if (data.status.length > 1) {
      $('#statusmenu').addClass('dropdown');
      var dropDownMenu = '<ul class="dropdown-menu">';
      for (var iloop = 0; iloop < data.status.length; iloop++) {
        dropDownMenu += '<li><a href="status.html?activePage=' + iloop + '">' + eval(data.status[iloop]) + '</a></li>';
      }
      dropDownMenu += '</ul>';
      $('#statuslink').html('Status <b class="caret"></b>')
      $(dropDownMenu).insertAfter('#statuslink');
      $('#statuslink').addClass('dropdown-toggle');
      $('#statuslink').attr('data-toggle', 'dropdown');
      $('#statuslink').attr('href', '#');
    }
  }

  if (data.statistics == undefined) {
    $('#statisticsmenu').addClass('hide');
  }
  else {
    if (data.statistics.length > 1) {
      $('#statisticsmenu').addClass('dropdown');
      var dropDownMenu = '<ul class="dropdown-menu">';
      for (var iloop = 0; iloop < data.statistics.length; iloop++) {
        dropDownMenu += '<li><a href="statistics.html?activePage=' + iloop + '">' + eval(data.statistics[iloop]) + '</a></li>';
      }
      dropDownMenu += '</ul>';
      $('#statisticslink').html('Statistics <b class="caret"></b>')
      $(dropDownMenu).insertAfter('#statisticslink');
      $('#statisticslink').addClass('dropdown-toggle');
      $('#statisticslink').attr('data-toggle', 'dropdown');
      $('#statisticslink').attr('href', '#');
    }
  }

  if (data.addons != undefined) {
    if (data.addons.length > 0) {
      $('#addonsmenu').removeClass('hide');
      $('#addonslink').html(eval(data.addons[0]));
    }
    if (data.addons.length > 1) {
      $('#addonsmenu').addClass('dropdown');
      var dropDownMenu = '<ul class="dropdown-menu">';
      for (var iloop = 0; iloop < data.addons.length; iloop++) {
        dropDownMenu += '<li><a href="addons.html?activePage=' + iloop + '">' + eval(data.addons[iloop]) + '</a></li>';
      }
      dropDownMenu += '</ul>';
      $('#addonslink').html('Add-ons <b class="caret"></b>')
      $(dropDownMenu).insertAfter('#addonslink');
      $('#addonslink').addClass('dropdown-toggle');
      $('#addonslink').attr('data-toggle', 'dropdown');
      $('#addonslink').attr('href', '#');
    }
  }
}

function getVersion() {
  $.ajax({
    url: 'version.json',
    dataType: 'json',
    async: false,
    success: function (data) {
      localStorage.setItem('version', data.version);
    }
  })
}

$(function () {

  if (localStorage == null) {
    alert("TypeError: localStorage is null\n\n" +
      "Activate HTML5 localStorage before continuing."
    );
  }

  // Construct the page template
  getVersion();
  AddTopmenu();
  AddDialogs();
  AddFooter();
  UpdateMenu();

});
